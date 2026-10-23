# Upstream PR: atomic queue edits and durable addressed input admission

Target: `tinyhumansai/tinyagents` (canonical repository in the owner's manifest).
Published personal-fork branch: `codex/run-queue-durable-input-upstream`.
PR: https://github.com/tinyhumansai/tinyagents/pull/567
Head: `f4e620d66`. All remote CI checks passed; ready for review.

## Summary

- Add atomic first-match edit/move operations to RunQueue, preserving lane and
  delivery order while reporting an already-consumed message as not found.
- Add an optional RunInboxStore capability on scoped DriverRunLedger handles.
  Validate exact persisted parent lineage and commit input, deduplication
  identity, event sequence and queued receipt in one checked batch.
- Keep receipt meaning explicit: durably queued, never model delivery or
  successful resumption. This foundation does not launch workers or implement
  recipient consumption/acknowledgement.
- Declare the new inputs collection at its initial version 1; preserve existing
  ledger/history migration contracts and refuse unsupported atomic backends.

## Local verification

The checks below passed on OpenHuman's pinned owner base. Current upstream moved
to the storage-port contracts afterward. A separate local worktree at
`target/owner-upstream-tinyagents` now ports these two commits onto upstream
`8dd8f3ffd584cfd8bb2ecbce8526e85e75dad5b2`, preserving its import APIs and using
`tinyagents-storage-kit` plus the owner's testkit.

Current-upstream verification: 72 ledger-driver tests, 26 queue tests,
whole-workspace formatting and scoped all-target Clippy for harness/session
with warnings denied all passed. The older counts below describe the pinned
base, not the published branch.

- 83 ledger-driver tests passed, including three new inbox cases, existing
  migration checks, scope isolation and native backend failure behavior.
- The queue mutation focused suite passed 25 tests and passed its latest rerun.
- Owner cargo check passed. Rust source review found no concrete inbox, queue or
  event atomicity defect; runtime integration is outside this draft's scope.

Local pinned-base commits: `7583b96` (queue), `2db1d19` (inbox).
Local current-upstream commits: `356b192f0` (queue), `029657413` (inbox).

## Changed owner areas

- `crates/tinyagents-harness/src/run_queue/`: API, documentation and regressions.
- `crates/tinyagents-session/src/run_ledger/inbox.rs`: addressed input port/types.
- `crates/tinyagents-session/src/run_ledger/drivers/inbox.rs`: checked admission.
- Ledger driver schema/events/storage declarations and focused regressions.

## Follow-on work

Recipient integration must commit durable input consumption and delivery state,
then connect execution-backed continuation and shared budgets. OpenHuman must
wait for upstream availability before consuming the changed gitlink and wiring
those contracts to its approved run panel.
