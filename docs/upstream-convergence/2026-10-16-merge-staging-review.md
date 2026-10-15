# OpenHuman upstream integration — staging review (2026-10-16)

**NOT READY TO MERGE.** This staging branch exists to preserve the exact original Git histories while auditing how to combine their working trees.

- Fork main: `7070ff332c9a69032e22a0b6c7431eea462e162d` (754 fork-exclusive commits).
- Upstream main at staging: `b1693361dfb696311d47b400f6a15ad376140179`.
- Original merge base: `ccd8fb2411eada7bd41a1a6fcf7f303f04344f00`.
- Safety ref: `backup/main-pre-upstream-sync-20261015-7070ff33`.
- Tree comparison: 259 fork-only changes, 1,502 upstream-only changes, 8 identical changes, 54 divergent paths.
- 35 divergent text files had nonoverlapping line-level edits from each side and were mechanically combined. The merge algorithm passed its 10 local algorithm unit tests, **not** the application test suite.
- The remaining 17 divergent paths temporarily retain their fork blobs in the staging tree, to avoid discarding custom functionality. They are unresolved. Their upstream changes must be ported back in before this branch can replace `main`. The restored `harness_assembly.rs` may need to be removed after relocating its changes.

## Reconciled after the initial merge staging

- `crates/openhuman-core/src/agent/tinyagents/live_harness.rs`: used upstream host-tool registration, DispatchCtx-aware hooks, new 4-argument ToolPolicyMiddleware, and relocated middleware imports. Ported the fork's session permission precheck with `ApprovalSecurityMiddleware::with_session_permission` to prevent prompt-then-deny regressions. Existing middleware approval tests cover allowed/denied/ceiling outcomes, but the integrated source has not been compiled.
- `crates/openhuman-core/src/agent/tinyagents/verify_before_finish.rs`: retained fork's explicit single-round mutation/delegation verification triggers (upstream's simplified `rounds_or_called` omits them) and adopted upstream `tinyagents_harness::middleware::wrap_harness_instruction` import. Existing fork test `single_round_mutations_and_delegations_require_finish_verification` guards regression, but the integrated source has not been compiled.

## Unresolved paths

1. `crates/openhuman-core/src/agent/tinyagents/harness_assembly.rs` — upstream deleted/refactored; port fork-specific harness assembly behavior into the replacement architecture.
2. `crates/openhuman-core/src/modules/ops.rs` — preserve raced module admission behavior and adopt upstream resolver/registry changes.
3. `crates/openhuman-core/src/core/observability.rs` — reconcile quota predicate visibility and conditional compilation.
4. `crates/openhuman-core/src/tools/impl/system/shell.rs` — integrate verification/security with upstream shell-progress instrumentation.
5. `crates/openhuman-core/src/security/credentials/schemas.rs` — preserve activation handlers while propagating new DispatchCtx.
6. `crates/openhuman-tinyhumans/src/jev/ranker.rs` — integrate fork System-One circuit breaker with chat-scoped disclosure/approval routing.
7. `crates/openhuman-tinyhumans/src/jev/ranker_tests.rs` — combine both regression suites.
8. `crates/openhuman-cli/Cargo.toml` — harmonize feature-gated E2E target declarations.
9. `crates/openhuman-core/src/media/generation/artifact_tool_tests.rs` — combine artifact persistence and ready-state assertions.
10. `crates/openhuman-core/src/config/ops_files_dir_tests.rs` — reconcile equivalent legacy migration fixtures.
11. `tests/tinymcp_storage_smoke_e2e.rs` — combine network-isolation and fixture preconditions.
12. `scripts/test-rust-e2e.sh` — preserve module-staged feature-gated integration tests.
13. `scripts/e2e-core.sh` — run gated TinyMCP E2E suite without duplicate/omitted execution.
14. `scripts/kernel-floor.limits` — regenerate dependency floor from final Cargo tree.
15. `scripts/ci/check-dep-sim-calibration.sh` — recompute simulator baselines from final graph.
16. `scripts/ci/check-openhuman-rust-layout.mjs` — combine current upstream file-size ratchets with fork-specific limits.
17. `scripts/ci/process-statics.json` — regenerate after code integration.

## Release gates

Do not update `main` or claim completion until all 17 remaining paths have been reconciled, the integrated tree compiles, lint/type checks pass, Rust core/security/inference/module/shell tests pass, provider fleet and UI tests pass, and integration/E2E tests execute on supported runners. Confirm latest upstream remains an ancestor immediately before promotion. Preserve any substantive upstream improvement even when its refactor touches fork-specific behavior.

**Important:** Git ancestry containing both parents is not proof that all upstream code survives in the resulting tree; the 19 temporary fork-biased resolutions require explicit follow-up.
