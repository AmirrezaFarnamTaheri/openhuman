use super::{runtime, CALLBACKS, RUNTIME};
use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::oneshot;

struct Gate {
    entered: oneshot::Sender<()>,
    release: oneshot::Receiver<()>,
}

struct ProbeState {
    starts: AtomicUsize,
    served: AtomicUsize,
    gate: Mutex<Option<Gate>>,
}

static PROBE: Mutex<Option<Arc<ProbeState>>> = Mutex::new(None);

struct Probe {
    state: Arc<ProbeState>,
}

impl Drop for Probe {
    fn drop(&mut self) {
        PROBE
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
    }
}

fn current_probe() -> Option<Arc<ProbeState>> {
    PROBE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

fn arm() -> (Probe, oneshot::Receiver<()>, oneshot::Sender<()>) {
    assert!(RUNTIME.get().is_none(), "the child must start a cold runtime");
    assert!(CALLBACKS.get().is_none(), "callbacks must not be initialized");
    let (entered_tx, entered_rx) = oneshot::channel();
    let (release_tx, release_rx) = oneshot::channel();
    let state = Arc::new(ProbeState {
        starts: AtomicUsize::new(0),
        served: AtomicUsize::new(0),
        gate: Mutex::new(Some(Gate {
            entered: entered_tx,
            release: release_rx,
        })),
    });
    assert!(PROBE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .replace(Arc::clone(&state))
        .is_none());
    (Probe { state }, entered_rx, release_tx)
}

pub(super) fn before_install() {
    if let Some(state) = current_probe() {
        state.starts.fetch_add(1, Ordering::SeqCst);
    }
}

pub(super) async fn after_install() {
    let Some(state) = current_probe() else {
        return;
    };
    state.served.fetch_add(1, Ordering::SeqCst);
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

fn owner_runtime<F, T>(future: F) -> oneshot::Receiver<T>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let (done_tx, done_rx) = oneshot::channel();
    let owner = std::thread::Builder::new()
        .name("module-callback-owner".into())
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
    // The isolated child must be able to fail on the coordinator's timeout
    // even when its owner is deliberately blocked in the storage bridge.
    drop(owner);
    done_rx
}

#[tokio::test]
async fn synchronous_retry_does_not_block_callback_owner() {
    if crate::util::module_test_process::run(
        module_path!(),
        "synchronous_retry_does_not_block_callback_owner",
    ) {
        return;
    }
    tokio::time::timeout(Duration::from_secs(20), async {
        let (probe, entered, release) = arm();
        let (start_sync_tx, start_sync_rx) = oneshot::channel();
        let (polled_tx, polled_rx) = oneshot::channel();
        let finished = owner_runtime(async move {
            let initial = tokio::spawn(runtime());
            start_sync_rx.await.expect("coordinator starts sync retry");
            let retry = crate::storage::block_on_anyhow(async move {
                // Release the installer only after this future is polled on
                // the bridge and the owner's runtime is synchronously blocked.
                let _ = polled_tx.send(());
                runtime().await.map_err(|error| anyhow::anyhow!("{error}"))
            })
            .expect("synchronous retry finishes");
            let initial = initial
                .await
                .expect("initial caller joins")
                .expect("initial callback installation finishes");
            (initial, retry)
        });

        entered.await.expect("all callbacks served before publication");
        start_sync_tx.send(()).expect("owner remains available");
        polled_rx.await.expect("sync bridge future was polled");
        release.send(()).expect("callback gate remains live");
        let concurrent = runtime()
            .await
            .expect("independent caller retrieves the module runtime");
        let (initial, synchronous) = finished.await.expect("owner runtime shut down");
        assert!(std::ptr::eq(initial, synchronous));
        assert!(std::ptr::eq(initial, concurrent));
        assert!(matches!(CALLBACKS.get(), Some(Ok(()))));
        assert_eq!(probe.state.starts.load(Ordering::SeqCst), 1);
        assert_eq!(probe.state.served.load(Ordering::SeqCst), 1);
    })
    .await
    .expect("module callback publication must finish without the blocked caller runtime");
}

#[tokio::test]
async fn cancelled_caller_runtime_keeps_one_callback_install() {
    if crate::util::module_test_process::run(
        module_path!(),
        "cancelled_caller_runtime_keeps_one_callback_install",
    ) {
        return;
    }
    tokio::time::timeout(Duration::from_secs(20), async {
        let (probe, entered, release) = arm();
        let (cancel_tx, cancel_rx) = oneshot::channel();
        let stopped = owner_runtime(async move {
            let caller = tokio::spawn(runtime());
            cancel_rx.await.expect("coordinator cancels the caller");
            caller.abort();
            let error = caller.await.err().expect("caller must be cancelled");
            assert!(error.is_cancelled());
        });

        entered.await.expect("all callbacks served before cancellation");
        cancel_tx.send(()).expect("owner remains available");
        stopped.await.expect("cancelled caller runtime was dropped");
        release
            .send(())
            .expect("callback installation survives its caller runtime");
        // Check installation count before unwrapping the retry: a duplicate
        // installation may itself fail, but that must not obscure the defect.
        let first = runtime().await;
        let starts = probe.state.starts.load(Ordering::SeqCst);
        assert_eq!(starts, 1, "cancelled caller must not reinstall module callbacks");
        let first = first.expect("detached callback installation committed its outcome");
        let second = runtime().await.expect("cached retry succeeds");
        assert!(std::ptr::eq(first, second));
        assert!(matches!(CALLBACKS.get(), Some(Ok(()))));
        assert_eq!(probe.state.served.load(Ordering::SeqCst), 1);
    })
    .await
    .expect("callback installation must survive cancellation and caller runtime shutdown");
}
