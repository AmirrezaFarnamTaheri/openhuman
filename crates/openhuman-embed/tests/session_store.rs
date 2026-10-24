//! End-to-end proof that a host session store owns every conversation.
//!
//! A stateless runtime with an [`InMemorySessionStores`] runs two agents
//! against a `wiremock` provider. Their transcripts and journal must land in
//! the store, and their run status in the runtime's in-memory storage, each
//! agent's apart from the other's; a reopened
//! agent must resume its thread *from the store*; and no conversation data may
//! be written under the runtime's scratch workspace.
//!
//! One `#[test]` because a runtime claims a process-wide slot.

mod common;

use std::collections::HashSet;
use std::sync::Arc;

use common::{chat_requests, offline_config, provider, runtime, stub_backend};
use openhuman_core::storage::DocumentStoreExt as _;
use openhuman_embed::session_store::session_store_conformance;
use openhuman_embed::{
    Access, AgentSpec, InMemorySessionStores, Provider, Runtime, RuntimeError,
    SessionStoreProvider, Workspace,
};

/// Every path under `root`, relative to it.
fn files_under(root: &std::path::Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending.push(path.clone());
            }
            found.push(
                path.strip_prefix(root)
                    .unwrap_or(&path)
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    found
}

/// Flow resources are initialized outside the first turn's critical path.
/// Wait for their published snapshot before recording the filesystem baseline;
/// a successful turn alone does not mean this legitimate materialization ended.
async fn wait_for_flow_resources(
    runtime: &Runtime,
    config: &openhuman_core::config::Config,
    agent: Option<&str>,
) {
    let compiled = openhuman_core::core::runtime::compiled_features();
    if !["skills", "flows", "modules"]
        .iter()
        .all(|feature| compiled[*feature])
        || !config.modules.enabled
        || !config.components.skills
        || !config.components.flows
        || !config.components.tinyflows
    {
        return;
    }
    let context = runtime.core_runtime().context();
    let context = agent.map_or_else(|| Arc::clone(context), |id| context.for_agent(id));
    let dispatch = openhuman_core::core::runtime::DispatchCtx::new(context);
    if dispatch
        .domains()
        .is_some_and(|domains| !domains.skills || !domains.flows)
    {
        return;
    }
    tokio::time::timeout(std::time::Duration::from_secs(90), async {
        loop {
            if openhuman_core::skills::load_workflow_metadata_for(&dispatch, config)
                .iter()
                .any(|skill| skill.id() == "flow-authoring")
            {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    })
    .await
    .unwrap_or_else(|_| {
        eprintln!("session_store: flow resources did not initialize for {agent:?}");
        panic!("flow resource initialization timed out");
    });
}

/// Even the initial baseline must not hide an accidentally persisted transcript,
/// turn journal or session database.
fn assert_no_durable_sessions(paths: &[String]) {
    let durable: Vec<_> = paths
        .iter()
        .filter(|path| {
            path.split(['/', '\\']).any(|part| {
                matches!(
                    part,
                    "session_raw" | "session_db" | "tinyagents_store" | "turn_states"
                )
            })
        })
        .collect();
    if !durable.is_empty() {
        eprintln!("session_store: durable session paths found: {durable:?}");
    }
    assert!(
        durable.is_empty(),
        "session data must stay in the host store"
    );
}

#[test]
fn a_stateless_runtime_keeps_every_conversation_in_its_session_store() {
    let _ = env_logger::builder().is_test(true).try_init();
    runtime().block_on(async {
        tokio::spawn(async move {
            // No store, no stateless runtime — and the failed build releases
            // the process slot, so the next build below succeeds.
            let refused = Runtime::builder()
                .config(offline_config())
                .workspace(Workspace::stateless())
                .build()
                .await;
            assert!(
                matches!(refused, Err(RuntimeError::NoSessionStore)),
                "a stateless runtime needs a session store"
            );

            let backend = stub_backend().await;
            let model = provider("noted").await;
            let store = Arc::new(InMemorySessionStores::new());
            let mut config = offline_config();
            // On by default in the product: it must stand down by itself once
            // a host store owns the transcripts.
            config.agent.session_dual_write = true;
            config.modules.allow_download = false;
            let runtime = Runtime::builder()
                .config(config)
                .workspace(Workspace::stateless())
                .backend_url(backend.uri())
                .session_store(store.clone())
                .build()
                .await
                .expect("a stateless runtime with a store builds");
            assert!(openhuman_core::storage::sessions::is_installed());
            let scratch = runtime.root_dir().to_path_buf();

            let spec = |id: &str| {
                AgentSpec::new(id)
                    .provider(
                        Provider::openai_compatible(format!("{}/v1", model.uri()), "sk-test")
                            .model("store-model"),
                    )
                    .access(Access::readonly())
            };

            // ── asha: two turns on one thread ─────────────────────────────
            let asha = runtime.agent(spec("asha")).expect("asha instantiates");
            let ravi = runtime.agent(spec("ravi")).expect("ravi instantiates");
            let first = asha
                .turn("remember the table is for two")
                .send()
                .await
                .expect("first turn");
            assert_eq!(first.reply, "noted");
            let thread = first.session_id.clone();
            ravi.turn("hello").send().await.expect("ravi's first turn");
            let core_config = runtime
                .core_runtime()
                .context()
                .embedder_config()
                .expect("the runtime retains its supplied config");
            wait_for_flow_resources(&runtime, core_config, None).await;
            wait_for_flow_resources(&runtime, asha.config(), Some("asha")).await;
            wait_for_flow_resources(&runtime, ravi.config(), Some("ravi")).await;
            // Both agents' first turns and their lazy resources are now
            // initialized. Reject durable session data before accepting this
            // scaffolding, then require every later turn to add no files.
            let baseline = files_under(&scratch);
            assert_no_durable_sessions(&baseline);
            let baseline: HashSet<_> = baseline.into_iter().collect();
            asha.turn("and at eight")
                .session(&thread)
                .send()
                .await
                .expect("second turn");
            // Reusing an id waits for the old instance's cleanup: dropping a
            // handle only schedules its MCP close, and the id stays reserved
            // until that close is acknowledged (`CleanupPending`). Removing
            // awaits the close, so the reopen below is deterministic.
            runtime
                .remove_agent("asha")
                .await
                .expect("asha is removed and its cleanup acknowledged");
            drop(asha);

            // ── reopened, asha resumes from the store ─────────────────────
            let asha = runtime.agent(spec("asha")).expect("asha reopens");
            asha.turn("what did I say?")
                .session(&thread)
                .send()
                .await
                .expect("resumed turn");
            let requests = chat_requests(&model).await;
            let resumed =
                String::from_utf8_lossy(&requests.last().expect("a request").body).into_owned();
            assert!(
                resumed.contains("remember the table is for two")
                    && resumed.contains("and at eight"),
                "the reopened agent replays its thread from the store"
            );

            // ── ravi: a different agent ───────────────────────────────────
            ravi.turn("hello again")
                .send()
                .await
                .expect("ravi's next turn");

            // Asha's transcript is in her stores, and only hers.
            let asha_stores = store.for_agent("asha");
            let transcript = asha_stores
                .transcripts
                .root_for_thread(&thread)
                .expect("asha's thread is in her store")
                .read_session()
                .expect("readable")
                .expect("written");
            assert!(transcript
                .messages
                .iter()
                .any(|message| message.content.contains("table is for two")));
            assert!(
                store
                    .for_agent("ravi")
                    .transcripts
                    .root_for_thread(&thread)
                    .is_none(),
                "another agent cannot reach asha's thread"
            );
            // The turn journal's run status went to the agent's own scope of
            // the runtime's storage, which a stateless runtime keeps in memory.
            let storage = openhuman_core::storage::installed()
                .expect("a stateless runtime installs a storage backend");
            assert_eq!(storage.driver(), "memory");
            let runs = storage
                .for_scope(&openhuman_core::storage::scope_for_agent("asha"))
                .expect("asha's storage scope")
                .documents()
                .query_all(
                    tinyagents_harness::observability::DriverStatusStore::COLLECTION,
                    &Default::default(),
                )
                .await
                .expect("status records list");
            assert!(!runs.is_empty(), "asha's runs are recorded in her scope");

            // Nothing durable reached the scratch workspace.
            let after = files_under(&scratch);
            assert_no_durable_sessions(&after);
            let written: Vec<_> = after
                .into_iter()
                .filter(|path| !baseline.contains(path))
                .collect();
            if !written.is_empty() {
                eprintln!("session_store: unexpected workspace writes: {written:?}");
            }
            assert!(
                written.is_empty(),
                "later turns must not add workspace files, found: {written:?}"
            );

            // Dropping a handle only schedules its MCP close, and the deferred
            // close keeps the scratch workspace until it is acknowledged, so
            // remove both agents (which awaits the close) before the runtime
            // goes.
            for agent in ["asha", "ravi"] {
                runtime
                    .remove_agent(agent)
                    .await
                    .expect("agent is removed and its cleanup acknowledged");
            }
            drop((asha, ravi));
            drop(runtime);
            assert!(
                !openhuman_core::storage::sessions::is_installed(),
                "the store is removed with the runtime"
            );
            assert!(!scratch.exists(), "the scratch workspace is removed");

            // The store a runtime used still meets the port's contract.
            session_store_conformance(store.as_ref()).await;
        })
        .await
        .expect("test task");
    });
}
