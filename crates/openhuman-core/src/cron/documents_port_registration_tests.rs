use super::*;
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tinyflows_bus::schedule::{DOCUMENT_INTERFACE, DOCUMENT_METHOD};
use tokio::sync::oneshot;

struct Gate {
    entered: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}

struct ProbeState {
    serves: AtomicUsize,
    gate: Mutex<Option<Gate>>,
}

static PROBES: LazyLock<Mutex<HashMap<String, Arc<ProbeState>>>> = LazyLock::new(Default::default);

struct Probe {
    path: String,
    state: Arc<ProbeState>,
}

impl Drop for Probe {
    fn drop(&mut self) {
        PROBES
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.path);
    }
}

fn arm(scope: &DocScope) -> (Probe, oneshot::Receiver<()>, oneshot::Sender<()>) {
    let path = path_for(&scope.key());
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let state = Arc::new(ProbeState {
        serves: AtomicUsize::new(0),
        gate: Mutex::new(Some(Gate {
            entered: entered_tx,
            release: release_rx,
        })),
    });
    assert!(PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(path.clone(), Arc::clone(&state))
        .is_none());
    (Probe { path, state }, entered_rx, release_tx)
}

pub(super) async fn after_serve(path: &str) {
    let state = PROBES
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(path)
        .cloned();
    let Some(state) = state else {
        return;
    };
    state.serves.fetch_add(1, Ordering::SeqCst);
    let gate = state
        .gate
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .take();
    if let Some(gate) = gate {
        let _ = gate.entered.send(());
        let _ = gate.release.await;
    }
}

fn fixture() -> (tempfile::TempDir, Config, DocScope) {
    let workspace = tempfile::tempdir().expect("isolated workspace");
    let config = Config {
        workspace_dir: workspace.path().to_path_buf(),
        ..Config::default()
    };
    let backend: Arc<dyn StorageBackend> = Arc::new(crate::storage::MemoryStorage::new());
    let scope = DocScope::on_backend(&config, backend, Scope::local());
    (workspace, config, scope)
}

fn owner_runtime<F, T>(future: F) -> oneshot::Receiver<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (done_tx, done_rx) = oneshot::channel();
    let owner = std::thread::Builder::new()
        .name("cron-registration-owner".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("owner runtime");
            let result = runtime.block_on(future);
            drop(runtime);
            let _ = done_tx.send(result);
        })
        .expect("owner thread");
    // These tests run in a child process: a deliberately blocked owner cannot
    // prevent the coordinator's timeout from failing the child and exiting.
    drop(owner);
    done_rx
}

fn same_address(left: &CallbackAddress, right: &CallbackAddress) {
    assert_eq!(left.destination, right.destination);
    assert_eq!(left.object_path, right.object_path);
}

async fn probe_callback(callback: &CallbackAddress) {
    let runtime = crate::modules::runtime::runtime()
        .await
        .expect("module runtime remains available");
    let proxy = runtime
        .connection()
        .proxy(
            &callback.destination,
            &callback.object_path,
            DOCUMENT_INTERFACE,
        )
        .expect("document callback proxy");
    let reply: Value = proxy
        .call(DOCUMENT_METHOD, (json!({ "op": "capabilities" }),))
        .await
        .expect("cached callback remains served");
    assert!(reply["value"]["transactions"].is_boolean(), "{reply}");
    assert_eq!(reply["value"]["record_metadata"], true, "{reply}");
}

#[tokio::test]
async fn synchronous_retry_does_not_block_registration_owner() {
    if crate::util::module_test_process::run(
        module_path!(),
        "synchronous_retry_does_not_block_registration_owner",
    ) {
        return;
    }
    tokio::time::timeout(Duration::from_secs(20), async {
        let (_workspace, config, scope) = fixture();
        module::proxy(&config)
            .await
            .expect("linked module attaches");
        let (probe, entered, release) = arm(&scope);
        let (start_sync_tx, start_sync_rx) = oneshot::channel();
        let (polled_tx, polled_rx) = oneshot::channel();
        let owner_scope = scope.clone();
        let finished = owner_runtime(async move {
            let initial_scope = owner_scope.clone();
            let initial = tokio::spawn(async move { address(&initial_scope).await });
            start_sync_rx.await.expect("coordinator starts sync retry");
            let retry = crate::storage::block_on_anyhow(async move {
                // Only this first-poll signal permits the coordinator to
                // release the initializer while its caller is in the bridge.
                let _ = polled_tx.send(());
                address(&owner_scope).await
            })
            .expect("synchronous retry finishes");
            let initial = initial
                .await
                .expect("initial caller joins")
                .expect("initial registration finishes");
            (initial, retry)
        });

        entered
            .await
            .expect("initial callback served before commit");
        start_sync_tx.send(()).expect("owner remains available");
        polled_rx.await.expect("sync bridge future was polled");
        release.send(()).expect("registration gate remains live");
        let concurrent = address(&scope)
            .await
            .expect("independent runtime retrieves the same registration");
        let (initial, synchronous) = finished.await.expect("owner runtime shut down");
        same_address(&initial, &synchronous);
        same_address(&initial, &concurrent);
        assert_eq!(initial.object_path, probe.path);
        assert_eq!(
            probe.state.serves.load(Ordering::SeqCst),
            1,
            "concurrent callers must not replace the served callback"
        );
        probe_callback(&concurrent).await;
    })
    .await
    .expect("registration must finish without the blocked caller runtime");
}

#[tokio::test]
async fn cancelled_caller_runtime_keeps_one_registration() {
    if crate::util::module_test_process::run(
        module_path!(),
        "cancelled_caller_runtime_keeps_one_registration",
    ) {
        return;
    }
    tokio::time::timeout(Duration::from_secs(20), async {
        let (_workspace, config, scope) = fixture();
        module::proxy(&config)
            .await
            .expect("linked module attaches");
        let (probe, entered, release) = arm(&scope);
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let owner_scope = scope.clone();
        let stopped = owner_runtime(async move {
            let caller = tokio::spawn(async move { address(&owner_scope).await });
            cancel_rx.await.expect("coordinator cancels the caller");
            caller.abort();
            let error = caller.await.err().expect("caller must be cancelled");
            assert!(error.is_cancelled());
        });

        entered
            .await
            .expect("callback served before caller cancellation");
        cancel_tx.send(()).expect("owner remains available");
        stopped.await.expect("cancelled caller runtime was dropped");
        release
            .send(())
            .expect("registration survives its caller runtime");
        let first = address(&scope)
            .await
            .expect("detached registration committed its cache entry");
        let second = address(&scope).await.expect("cached retry succeeds");
        same_address(&first, &second);
        assert_eq!(first.object_path, probe.path);
        assert_eq!(
            probe.state.serves.load(Ordering::SeqCst),
            1,
            "cancelled caller must not require serving a replacement"
        );
        probe_callback(&second).await;
    })
    .await
    .expect("registration must survive cancellation and caller runtime shutdown");
}
