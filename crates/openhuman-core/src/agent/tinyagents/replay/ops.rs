//! Read-only business logic for the agent replay/status RPC surface
//! (workstream 05.x): a workspace-to-store adapter over the upstream readers in
//! `tinyagents_harness::observability::replay`. Every function opens the same
//! journal and run-status stores (in the storage backend) the turn journal writes
//! and never writes, mutates, or bypasses any security/approval/sandbox gate.

use std::path::Path;

use tinyagents_harness::events::HarnessRunStatus;
use tinyagents_harness::observability::replay::{self, RunEventsPage};
use tinyagents_harness::observability::StoreEventJournal;

/// Paged late-attach replay reader over the durable journal under `workspace`.
pub(crate) async fn read_run_events_page(
    workspace: &Path,
    run_id: &str,
    offset: u64,
    limit: u64,
) -> anyhow::Result<RunEventsPage> {
    let journal =
        StoreEventJournal::new(crate::agent::session_store::stores_or_ephemeral(workspace).journal);
    replay::read_run_events_page(&journal, run_id, offset, limit)
        .await
        .map_err(|e| {
            anyhow::anyhow!("[agent] replay read_run_events_page failed run_id={run_id}: {e}")
        })
}

/// Latest durable status for `run_id`, or `None` when the run is unknown.
pub(crate) async fn read_run_status(
    workspace: &Path,
    run_id: &str,
) -> anyhow::Result<Option<HarnessRunStatus>> {
    let store = crate::agent::session_store::status_store_for(workspace);
    replay::read_run_status(&store, run_id)
        .await
        .map_err(|e| anyhow::anyhow!("[agent] replay read_run_status failed run_id={run_id}: {e}"))
}

/// Active runs, optionally filtered by `thread_id` and/or `root_run_id`.
pub(crate) async fn list_active_runs(
    workspace: &Path,
    thread_id: Option<&str>,
    root_run_id: Option<&str>,
) -> anyhow::Result<Vec<HarnessRunStatus>> {
    let store = crate::agent::session_store::status_store_for(workspace);
    replay::list_active_runs(&store, thread_id, root_run_id)
        .await
        .map_err(|e| anyhow::anyhow!("[agent] replay list_active_runs failed: {e}"))
}

/// Derive a bounded local-check report from the same durable event stream
/// exposed by run_events. Never report task_verified=true: journal facts do
/// not establish full code coverage, CI on the final commit, or hidden shell
/// side effects. Reading remains workspace-scoped and has no side effects.
pub(crate) async fn read_run_local_checks(
    workspace: &Path,
    run_id: &str,
) -> anyhow::Result<super::local_checks::LocalCheckReport> {
    const PAGE_SIZE: u64 = 256;
    const MAX_PAGES: usize = 12;
    let mut summary = super::local_checks::LocalCheckAccumulator::default();
    let mut cursor = 0_u64;
    for _ in 0..MAX_PAGES {
        let page = read_run_events_page(workspace, run_id, cursor, PAGE_SIZE).await?;
        for observed in &page.events {
            summary.observe(observed);
        }
        match page.next_offset {
            None => return Ok(summary.finish(run_id, true)),
            Some(next) if next > cursor => cursor = next,
            Some(_) => return Ok(summary.finish(run_id, false)),
        }
    }
    Ok(summary.finish(run_id, false))
}
