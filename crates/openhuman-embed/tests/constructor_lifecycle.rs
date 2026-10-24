//! A cancelled turn cannot release resources still used by its synchronous
//! session constructor, including when removal requests a purge.

mod common;

use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;

use openhuman_core::core::runtime::AgentContextRegistry;
use openhuman_embed::{
    Agent, AgentDefinitionSpec, AgentError, AgentSpec, AgentTurnOrigin, CoreError, HostTurnTools,
    Runtime, ToolScopeSpec, TurnOutcome, Workspace,
};
use tokio::sync::oneshot;

const WAIT: Duration = Duration::from_secs(20);

struct FactoryGate {
    release: Option<mpsc::Sender<()>>,
    entered: oneshot::Receiver<Option<AgentTurnOrigin>>,
    finishing: oneshot::Receiver<bool>,
}

impl FactoryGate {
    fn release(&mut self) {
        if let Some(release) = self.release.take() {
            let _ = release.send(());
        }
    }

    async fn entered(&mut self) {
        let origin = tokio::time::timeout(WAIT, &mut self.entered)
            .await
            .expect("factory entry deadline")
            .expect("factory entered");
        assert!(
            matches!(
                origin,
                Some(AgentTurnOrigin::WebChat { ref thread_id, ref client_id, .. })
                    if thread_id == "constructor-thread" && client_id == "constructor-client"
            ),
            "the blocking factory keeps the caller's origin: {origin:?}"
        );
    }

    async fn finish(&mut self) {
        self.release();
        assert!(
            tokio::time::timeout(WAIT, &mut self.finishing)
                .await
                .expect("factory exit deadline")
                .expect("factory finished"),
            "the constructor owns its home through its final instruction"
        );
    }
}

impl Drop for FactoryGate {
    fn drop(&mut self) {
        self.release();
    }
}

fn blocked_agent(
    runtime: &Runtime,
    id: &str,
    provider: &wiremock::MockServer,
) -> (Agent, FactoryGate) {
    let home = runtime.workspace_dir().join("agents").join(id);
    let (release, released) = mpsc::channel();
    let released = Mutex::new(released);
    let (entered, entered_rx) = oneshot::channel();
    let entered = Mutex::new(Some(entered));
    let (finishing, finishing_rx) = oneshot::channel();
    let finishing = Mutex::new(Some(finishing));
    let agent = runtime
        .agent(
            AgentSpec::new(id)
                .provider(common::route(provider, "constructor-lifecycle"))
                .definition(
                    AgentDefinitionSpec::new()
                        .bare_prompt("Answer once.")
                        .tools(ToolScopeSpec::HostOnly),
                )
                .tools(move |_| {
                    if let Some(entered) = entered.lock().unwrap().take() {
                        let _ = entered.send(openhuman_core::security::turn_origin::current());
                    }
                    released
                        .lock()
                        .unwrap()
                        .recv_timeout(Duration::from_secs(60))
                        .expect("test releases the synchronous factory");
                    if let Some(finishing) = finishing.lock().unwrap().take() {
                        let _ = finishing.send(home.is_dir());
                    }
                    HostTurnTools::advertised(Vec::new())
                }),
        )
        .expect("agent");
    (
        agent,
        FactoryGate {
            release: Some(release),
            entered: entered_rx,
            finishing: finishing_rx,
        },
    )
}

fn start_turn(agent: &Agent) -> tokio::task::JoinHandle<Result<TurnOutcome, CoreError>> {
    let turn = agent.turn("Answer once.").origin(AgentTurnOrigin::WebChat {
        thread_id: "constructor-thread".into(),
        client_id: "constructor-client".into(),
        request_id: None,
    });
    tokio::spawn(turn.send())
}

async fn removal_case(
    runtime: Arc<Runtime>,
    provider: &wiremock::MockServer,
    id: &'static str,
    abort_removal: bool,
) {
    let (agent, mut gate) = blocked_agent(&runtime, id, provider);
    let home = agent.home_dir().to_path_buf();
    let scope = AgentContextRegistry::get(id)
        .unwrap()
        .host_overrides()
        .unwrap()
        .approval_scope()
        .unwrap();
    let turn = start_turn(&agent);
    gate.entered().await;
    let removing_runtime = runtime.clone();
    let removing = tokio::spawn(async move {
        if abort_removal {
            removing_runtime.remove_agent(id).purge().await
        } else {
            removing_runtime.remove_agent(id).await
        }
    });
    common::eventually("the exact instance closes admission", || {
        scope.is_closed().then_some(())
    })
    .await;
    if abort_removal {
        removing.abort();
        assert!(tokio::time::timeout(WAIT, removing)
            .await
            .unwrap()
            .unwrap_err()
            .is_cancelled());
    } else {
        let result = tokio::time::timeout(WAIT, removing)
            .await
            .expect("removal wait is bounded")
            .expect("removal task");
        assert!(
            matches!(result, Err(AgentError::CleanupPending)),
            "{result:?}"
        );
        // Retrying a deferred removal can add purge intent without releasing
        // the home or creating a replacement instance prematurely.
        let retry = tokio::time::timeout(WAIT, async { runtime.remove_agent(id).purge().await })
            .await
            .expect("deferred removal retry is bounded");
        assert!(matches!(retry, Err(AgentError::CleanupPending)));
    }
    let outcome = tokio::time::timeout(WAIT, turn)
        .await
        .expect("removed turn ends")
        .expect("turn task");
    assert!(
        matches!(outcome, Err(CoreError::AgentRemoved { ref agent_id, .. }) if agent_id == id),
        "{outcome:?}"
    );
    let refused = tokio::time::timeout(WAIT, agent.turn("after removal").send())
        .await
        .expect("removed instance refuses new turns");
    assert!(
        matches!(refused, Err(CoreError::AgentRemoved { ref agent_id, .. }) if agent_id == id),
        "{refused:?}"
    );
    assert!(home.is_dir(), "the blocked constructor retains its home");
    assert!(AgentContextRegistry::get(id).is_some());
    assert!(runtime.agent_ids().iter().any(|reserved| reserved == id));
    assert!(matches!(
        runtime.agent(AgentSpec::new(id)),
        Err(AgentError::DuplicateId(ref duplicate)) if duplicate == id
    ));
    gate.finish().await;
    common::eventually(
        "constructor release permits purge and deregistration",
        || {
            (!home.exists()
                && AgentContextRegistry::get(id).is_none()
                && !runtime.agent_ids().iter().any(|reserved| reserved == id))
            .then_some(())
        },
    )
    .await;
    drop(agent);
}

#[test]
fn blocking_construction_retains_agent_and_workspace_cleanup_authority() {
    let _ = env_logger::builder().is_test(true).try_init();
    common::runtime().block_on(async {
        tokio::spawn(async {
            let backend = common::stub_backend().await;
            let provider = common::provider("unreachable").await;
            let runtime = Arc::new(
                Runtime::builder()
                    .config(common::offline_config())
                    .workspace(Workspace::Ephemeral)
                    .backend_url(backend.uri())
                    .build()
                    .await
                    .expect("runtime"),
            );
            removal_case(runtime.clone(), &provider, "aborted-construction", true).await;
            removal_case(runtime.clone(), &provider, "timed-out-construction", false).await;

            let (agent, mut gate) = blocked_agent(&runtime, "dropped-construction", &provider);
            let workspace = agent.workspace_dir().to_path_buf();
            let turn = start_turn(&agent);
            gate.entered().await;
            turn.abort();
            assert!(tokio::time::timeout(WAIT, turn)
                .await
                .unwrap()
                .unwrap_err()
                .is_cancelled());
            drop(agent);
            drop(runtime);
            assert!(
                workspace.is_dir(),
                "the constructor retains the ephemeral workspace"
            );
            gate.finish().await;
            common::eventually("last constructor release permits workspace cleanup", || {
                (!workspace.exists()).then_some(())
            })
            .await;
            assert!(common::chat_requests(&provider).await.is_empty());
        })
        .await
        .expect("test task");
    });
}
