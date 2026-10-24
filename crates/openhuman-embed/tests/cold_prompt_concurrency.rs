//! Cold prompt preparation must yield to asynchronous storage even when the
//! caller gives Tokio only one worker and one blocking thread.

mod common;

use common::gated_provider::GatedProvider;
use common::{offline_config, stub_backend};
use openhuman_embed::{Access, AgentSpec, Provider, Runtime, Workspace};
use std::time::Duration;

const AGENTS: usize = 100;
const REPLY: &str = "cold-prompts-completed";
const HANG_GUARD: Duration = Duration::from_secs(300);

/// A failed deadlock assertion must not wait forever while Tokio joins the
/// blocking task that caused the failure.
struct BoundedRuntime(Option<tokio::runtime::Runtime>);

impl Drop for BoundedRuntime {
    fn drop(&mut self) {
        if let Some(runtime) = self.0.take() {
            runtime.shutdown_timeout(Duration::from_secs(10));
        }
    }
}

#[test]
fn cold_prompts_complete_with_one_worker_and_one_blocking_thread() {
    let _ = env_logger::builder().is_test(true).try_init();
    let executor = BoundedRuntime(Some(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .worker_threads(1)
            .max_blocking_threads(1)
            .thread_stack_size(openhuman_core::core::runtime::AGENT_WORKER_STACK_BYTES)
            .build()
            .expect("single-worker runtime"),
    ));
    let executor = executor.0.as_ref().expect("runtime is alive");
    #[cfg(feature = "modules")]
    common::link_tinyflows(executor);

    // Poll the large agent futures on the configured worker stack. Keep the
    // deadline on the libtest thread: a timer in this executor cannot fire if
    // the sole worker and blocking thread are both starved by the regression.
    let scenario = executor.spawn(async {
        let backend = stub_backend().await;
        let provider = GatedProvider::start(REPLY).await;
        let mut config = offline_config();
        config.modules.allow_download = false;
        let runtime = Runtime::builder()
            .config(config)
            .workspace(Workspace::Ephemeral)
            .backend_url(backend.uri())
            .build()
            .await
            .expect("embedded runtime builds");

        // Every agent has its own untouched prompt cache. A warm-up turn
        // would miss the identity read that used to block inside prompt
        // preparation and starve the storage driver's blocking task.
        let agents: Vec<_> = (0..AGENTS)
            .map(|index| {
                runtime
                    .agent(
                        AgentSpec::new(format!("cold-{index}"))
                            .provider(
                                Provider::openai_compatible(
                                    format!("{}/v1", provider.uri()),
                                    "sk-test",
                                )
                                .model("cold-prompt-model"),
                            )
                            .access(Access::readonly()),
                    )
                    .expect("cold agent instantiates")
            })
            .collect();
        provider.hold_until(AGENTS);
        let mut turns = tokio::task::JoinSet::new();
        for agent in &agents {
            let agent = agent.clone();
            turns.spawn(async move { agent.turn("Answer once.").send().await });
        }
        while let Some(outcome) = turns.join_next().await {
            let outcome = outcome
                .expect("cold turn task")
                .expect("cold turn completes");
            assert_eq!(outcome.reply, REPLY);
        }
        assert!(provider.is_open());
        assert_eq!(provider.chat_requests(), AGENTS);
        assert_eq!(provider.peak_in_flight(), AGENTS);

        common::finish_runtime_agents(&runtime).await;
        drop(agents);
        drop(runtime);
    });
    let (finished, completion) = std::sync::mpsc::sync_channel(1);
    executor.spawn(async move {
        let _ = finished.send(scenario.await);
    });
    completion
        .recv_timeout(HANG_GUARD)
        .unwrap_or_else(|error| {
            // The linked module's panic hook omits assertion payloads.
            eprintln!(
                "cold prompt regression did not complete: {error}; all {AGENTS} \
                 cold turns must overlap with one worker and one blocking thread"
            );
            panic!("cold prompt preparation did not finish within {HANG_GUARD:?}");
        })
        .expect("cold-prompt test task");
}
