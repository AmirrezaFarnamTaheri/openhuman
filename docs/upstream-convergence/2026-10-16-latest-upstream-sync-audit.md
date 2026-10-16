# Upstream convergence refresh — October 16, 2026

## Status and exact revisions

**Candidate integration only. Do not merge to `main` until CI and application tests succeed.**

- Original fork `main`: `7070ff332c9a69032e22a0b6c7431eea462e162d` (754 original fork-exclusive commits, preserved as reachable ancestry).
- Protected original state: `backup/main-pre-upstream-sync-20261015-7070ff33`.
- Previous reconciliation staging: `775814373149480140f8dd6a87fff268f3ea75fc` (23 corrective commits after the first two-parent merge).
- Previous upstream parent: `b1693361dfb696311d47b400f6a15ad376140179`.
- New upstream integration target: `5be1b570df9bf8f08dafbfce997066ce92f07a56`.
- The new integration must retain **both** the previous staging and latest upstream as Git parents, and should report 0 behind relative to that upstream SHA.

A two-parent merge preserves each original fork commit's SHA; a history-rewriting `git rebase` would not. It may show more than 754 ahead, owing to the fork's later fixes and integration commits.

## Complete Git-tree inventory

Compared previous staging, its upstream merge base, the newest upstream and the synthesized candidate using complete recursive Git trees (not the GitHub compare API's truncated 300-file list):

| Classification relative to previous upstream merge base | Paths |
| --- | ---: |
| Same on both sides | 6,192 |
| Only staging changed | 291 |
| Only newer upstream changed | 495 |
| Both changed divergently | 15 |

Candidate Git tree contains 6,993 non-tree entries. Every one of the 6,192 unchanged entries and 291 staging-only and 495 upstream-only entries retains exactly the appropriate parent blob SHA and Git mode: **zero unexpected tree-object drift**. All 15 divergent paths were reviewed individually. This tree-level result is **not** a substitute for building, tests or semantic integration review.

## The 15 divergent paths

| Path | Reconciliation |
| --- | --- |
| `.github/workflows/test-reusable.yml` | Combined upstream secure private XDG module directory and fork's explicit Windows PowerShell long-path setup. |
| `crates/openhuman-core/src/agent/tinyagents/middleware/tool_policy.rs` | Preserved fork's session/tool enforcement, included upstream import cleanup. |
| `crates/openhuman-core/src/config/ops/model.rs` | Combined fork provider/model settings with upstream config snapshot import. |
| `crates/openhuman-core/src/inference/provider/factory.rs` | Staging already carries upstream's explicit-context exports in addition to its credential-fleet and tool adapters; retained staging. |
| `crates/openhuman-core/src/inference/provider/factory/cloud_slug.rs` | Merged upstream managed backend resolution path into fork's advanced fleet routing. |
| `crates/openhuman-core/src/inference/provider/factory_tests.rs` | Retained fork-specific provider tests and both explicit-context and managed-ledger regression modules. |
| `crates/openhuman-core/src/security/credentials/schemas.rs` | Kept activated-provider RPC/DispatchCtx integration and applied upstream cleanup. |
| `crates/openhuman-core/src/tools/impl/system/shell.rs` | Preserved shell-verification receipts and updated all upstream context-aware execution calls. |
| `crates/openhuman-core/src/tools/ops.rs` | Applied upstream media file-root initialization while retaining fork tools. |
| `docs/gitbooks/en/developing/embed/api-index.json` | Combined fork exports and upstream steering API names; JSON structure parsed successfully. |
| `llms-full.txt` | Combined fork documentation with upstream turn-steering documentation. |
| `scripts/__tests__/self-hosted-lanes.test.mjs` | Added upstream pinned Python 3.12/isolated PATH/jq checksum tests, retained fork's concurrent per-lane native-fixture regressions. |
| `scripts/ci/self-hosted/lanes-plan.mjs` | Preserved fork's stronger lane-isolated native-module paths; shared upstream sccache rollback functionality already present. |
| `scripts/test-rust-e2e.sh` | Added upstream isolated XDG directory and per-run encrypted-storage key while retaining staged TinyMCP E2E feature gate. |
| `tests/storage_desktop_upgrade_e2e.rs` | Combined explicit `DispatchCtx` calls with newest module import migration changes and fork upgrade tests. |

The newer upstream integration also incorporates updated Cargo lockfiles, media artifact changes, workflow steering, MCP UI, execution context, and the new native managed-ledger tests via upstream-only paths.

## Earlier 19-path architectural reconciliation

The first staging merge deliberately retained 19 fork-preferred files until semantic reconciliation. The follow-up staging history contains targeted changes for the harness replacement, session-denial middleware, provider-credential DispatchCtx, JEV ranker circuit breaker with chat-scoped consent, module single-flight loader, quota observability, sandbox shell progress and verification, dependency ratchets, and gated TinyMCP E2E. Review the staging commits from `ae4504bb809d` through `775814373149` for their original patches and tests.

## Verification ledger and release blockers

- **Verified:** original fork SHA remains protected; candidate tree classified in full; exact blob/mode preservation for all non-overlapping paths; no textual conflict markers in inspected merged files; merged API-index JSON parses; key security, E2E, and CI assertions inspected.
- **Not verified:** Rust compilation, frontend typecheck/lint, complete Rust and JS tests, generated-file drift, dependency-floor recalculation against current Cargo.lock, full E2E, runtime provider behavior and CI Gate. The checkout cannot access GitHub through DNS here; GitHub did not show recent Actions workflow runs for this staging branch.
- **Required before main promotion:** run focused provider/session/tool/module/memory tests, `pnpm test:scripts`, `pnpm rust:layout`, `pnpm typecheck`, `pnpm lint`, `cargo check`, Rust gated module/E2E, `pnpm test` and CI Gate. Recheck upstream HEAD immediately before promotion, and only move `main` after green results.

**Do not confuse zero-behind ancestry with successful functional convergence.** This record is an audit of a candidate integration, not a completed release verification.
