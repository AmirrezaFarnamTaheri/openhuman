//! The typed child prompt shares the main turn's asynchronous identity read.

use super::*;
use crate::agent::subagent_host::run_subagent;
use crate::core::runtime::{CoreContext, DomainSet};
use std::time::Duration;

#[test]
fn typed_subagent_prompt_completes_with_one_worker_and_one_blocking_thread() {
    let executor = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(1)
        .max_blocking_threads(1)
        .thread_stack_size(crate::core::runtime::AGENT_WORKER_STACK_BYTES)
        .build()
        .expect("bounded subagent executor");
    let scenario = executor.spawn(async {
        let root = tempfile::tempdir().expect("isolated child workspace");
        let workspace = root.path().join("workspace");
        std::fs::create_dir_all(&workspace).unwrap();
        let mut config = crate::config::Config {
            workspace_dir: workspace.clone(),
            action_dir: workspace.clone(),
            config_path: root.path().join("config.toml"),
            ..Default::default()
        };
        config.components.juice = false;
        config.modules.allow_download = false;
        let core = CoreContext::for_test_with_config(DomainSet::full(), config);
        let provider = ScriptedProvider::new(vec![text_response("cold child completed")]);
        let mut parent = make_parent(provider.clone(), vec![]);
        parent.workspace_dir = workspace;
        let mut options = with_parent(parent, SubagentRunOptions::default());
        options.run_context = options.run_context.with_core(Some(Arc::clone(&core)));

        let outcome = CoreContext::scope(core, async {
            run_subagent(
                &make_def_named_tools(&[]),
                "Return the requested answer.",
                options,
            )
            .await
        })
        .await
        .expect("a cold child prompt must yield while its storage read runs");
        assert_eq!(outcome.output, "cold child completed");
        assert_eq!(provider.captured.lock().len(), 1);
    });
    let (finished, completion) = std::sync::mpsc::sync_channel(1);
    executor.spawn(async move {
        let _ = finished.send(scenario.await);
    });

    // A Tokio timeout on this executor could itself starve under the old
    // block_in_place -> storage spawn_blocking cycle. Keep the deadline on
    // libtest's independent thread and bound shutdown before asserting.
    let result = completion.recv_timeout(Duration::from_secs(60));
    executor.shutdown_timeout(Duration::from_secs(5));
    result
        .expect("typed child prompt deadlocked the bounded executor")
        .expect("typed child prompt task panicked");
}
