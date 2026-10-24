//! Cold prompt preparation must yield to asynchronous storage even when the
//! caller gives Tokio only one worker and one blocking thread.

mod common;

use common::gated_provider::GatedProvider;
use common::{offline_config, stub_backend};
use openhuman_embed::{Access, AgentSpec, Provider, Runtime, Workspace};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

const AGENTS: usize = 100;
const REPLY: &str = "cold-prompts-completed";
const HANG_GUARD: Duration = Duration::from_secs(300);
const STAGES: [&str; 7] = [
    "starting fixtures",
    "building runtime",
    "creating agents",
    "running cold turns",
    "removing agents",
    "dropping runtime",
    "finishing fixtures",
];

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
    let stage = Arc::new(AtomicUsize::new(0));
    let completed_turns = Arc::new(AtomicUsize::new(0));
    let scenario_stage = Arc::clone(&stage);
    let scenario_completed_turns = Arc::clone(&completed_turns);
    let scenario = executor.spawn(async move {
        let backend = stub_backend().await;
        let provider = GatedProvider::start(REPLY).await;
        scenario_stage.store(1, Ordering::SeqCst);
        let mut config = offline_config();
        config.modules.allow_download = false;
        let runtime = Runtime::builder()
            .config(config)
            .workspace(Workspace::Ephemeral)
            .backend_url(backend.uri())
            .build()
            .await
            .expect("embedded runtime builds");
        scenario_stage.store(2, Ordering::SeqCst);

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
        scenario_stage.store(3, Ordering::SeqCst);
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
            scenario_completed_turns.fetch_add(1, Ordering::SeqCst);
        }
        assert!(provider.is_open());
        assert_eq!(provider.chat_requests(), AGENTS);
        assert_eq!(provider.peak_in_flight(), AGENTS);

        scenario_stage.store(4, Ordering::SeqCst);
        common::finish_runtime_agents(&runtime).await;
        scenario_stage.store(5, Ordering::SeqCst);
        drop(agents);
        drop(runtime);
        scenario_stage.store(6, Ordering::SeqCst);
    });
    let (finished, completion) = std::sync::mpsc::sync_channel(1);
    executor.spawn(async move {
        let _ = finished.send(scenario.await);
    });
    completion
        .recv_timeout(HANG_GUARD)
        .unwrap_or_else(|error| {
            // The linked module's panic hook omits assertion payloads.
            let phase = STAGES[stage.load(Ordering::SeqCst)];
            let completed = completed_turns.load(Ordering::SeqCst);
            eprintln!(
                "cold prompt regression did not complete: {error}; \
                 stage={phase}, completed_turns={completed}/{AGENTS}; all {AGENTS} \
                 cold turns must overlap with one worker and one blocking thread"
            );
            panic!("cold prompt preparation did not finish within {HANG_GUARD:?}");
        })
        .expect("cold-prompt test task");
}
