# Upstream branch, pull request, and fork integration audit

Audit date: 2026-10-14 (UTC+03:30). Target: `AmirrezaFarnamTaheri/openhuman`.
Upstream: `tinyhumansai/openhuman`; initial pinned upstream `8cb71dbc4075e07e81f8eba1e043ef78cf44491d`, followed by `ccd8fb2411eada7bd41a1a6fcf7f303f04344f00`.
Original fork main: `f6534cedd8c8e23413d1de57eca659bcfb7ea990`.
Integration branch: `sync/upstream-2026-10-14`. Review draft PR: https://github.com/AmirrezaFarnamTaheri/openhuman/pull/5

## Method and limitations

- Upstream has two discoverable branches: `main` and `release`; release is an ancestor of main and adds no independent commit beyond the merged mainline.
- History is preserved through two-parent merges; fork-specific credential pools, model preset editing, per-model vision routes, and translations are retained rather than overwritten by upstream.
- Open PR enumeration uses the GitHub REST API (`state=open&per_page=100`) and snapshots all returned PR heads; upstream merges can change this list during the audit.
- The public API reported roughly 4,126 forks while direct/nested enumeration returned 4,293 visible fork records during an initial workflow run. This is a moving network, not a fixed census.
- Recent-activity GitHub search is only a screening sample. Approximately 180 recent candidate fork default heads have been compared explicitly. This is NOT the complete fork network, and default-branch comparisons do not cover unpublished work on other branches.
- `scripts/ci/upstream_fork_audit.py` plus `.github/workflows/fork-ahead-census.yml` provide repeatable, read-only enumeration, pinned-commit comparison, 800-request budgets, checkpoint resume, separate ahead/behind counts, and a COMPLETE/INCOMPLETE report. Do not declare all forks inspected until that run completes without errors; the first run was invalid due to a three-dot URL validator bug (now fixed and regression tested).
- Every non-trivial imported behavior has accompanying regression tests where available. GitHub CI, not the existence of source commits, determines readiness. Leave PR draft until build, lint, tests, and security validation pass.

## Upstream PR triage

| Upstream PR | Area | Decision and rationale |
|---|---|---|
| [#7773](https://github.com/tinyhumansai/openhuman/pull/7773) | Tinyagents host refactor | Defer: 53-file harness rewrite, highly coupled; requires isolated complete test run |
| [#7771](https://github.com/tinyhumansai/openhuman/pull/7771) | Autonomous resilience | Defer: draft, 32-file policy/agent behavior change; isolate and prove safety |
| [#7770](https://github.com/tinyhumansai/openhuman/pull/7770) | Broad filesystem scan restriction | **Adapted and integrated** to current TinyBox shell analysis, with tests |
| [#7769](https://github.com/tinyhumansai/openhuman/pull/7769) | Thinking timer turn scope | Already present: both files' blob SHA identical to this PR head; skip redundant cherry-pick |
| [#7757](https://github.com/tinyhumansai/openhuman/pull/7757) | Stop inactive web turns | Defer: 38-file cancellation/progress refactor; overlaps #7724; needs independent concurrency tests |
| [#7724](https://github.com/tinyhumansai/openhuman/pull/7724) | Persist interrupted chat | **Integrated** via exact patch and tests; now upstream main also contains it |
| [#7699](https://github.com/tinyhumansai/openhuman/pull/7699) | Approval before denied tool calls | **Integrated** five-file patch, regression coverage |
| [#7521](https://github.com/tinyhumansai/openhuman/pull/7521) | MCP lifetime ownership | Defer: 23-file patch targets removed/moved MCP modules in current upstream, cannot safely graft verbatim |
| [#7520](https://github.com/tinyhumansai/openhuman/pull/7520) | Storage bypass ratchet | Defer: existing storage architecture replaced portions of checker; several target files absent |
| [#7476](https://github.com/tinyhumansai/openhuman/pull/7476) | Memory sync iteration | Obsolete: original front-drain no longer exists after delegated `tinymemory` module architecture |
| [#7367](https://github.com/tinyhumansai/openhuman/pull/7367) | Dedup-lookup optimization | **Adapted and integrated** into current document store, with regression tests |
| [#7361](https://github.com/tinyhumansai/openhuman/pull/7361) | E2E typecheck CI | **Integrated** E2E TypeScript check and two profile tests |
| [#7331](https://github.com/tinyhumansai/openhuman/pull/7331) | TinySecurity authorization migration | Defer: draft, 74-file security boundary rewrite; requires authorization regression and sandbox validation |
| [#7279](https://github.com/tinyhumansai/openhuman/pull/7279) | Keenable search | Defer: 44-file external provider integration, credentials/privacy and provider behavior need separate review |
| [#7248](https://github.com/tinyhumansai/openhuman/pull/7248) | Account secret path carve-out | High-priority follow-up: 34-file security boundary change, test against newer storage and agent modules first |
| [#7205](https://github.com/tinyhumansai/openhuman/pull/7205) | Inline MCP Apps | Defer: >100-file untrusted iframe/bridge feature; requires isolation, CSP, UI E2E and permission review |
| [#7177](https://github.com/tinyhumansai/openhuman/pull/7177) | LLMTR built-in preset | **Adapted and integrated** Rust + TS catalogs, picker logo, and tests |
| [#7047](https://github.com/tinyhumansai/openhuman/pull/7047) | Codex session import | Defer: draft with vendored `tinymemory` change; needs migration/storage tests |
| [#7040](https://github.com/tinyhumansai/openhuman/pull/7040) | Composer dictation | Defer: 42-file audio permission/UI feature, needs cross-platform accessibility and browser tests |
| [#7037](https://github.com/tinyhumansai/openhuman/pull/7037) | Rust toolchain sandbox mounts | Defer: old sandbox `grants.rs` and tests removed in current upstream; exact patch cannot apply |
| [#7020](https://github.com/tinyhumansai/openhuman/pull/7020) | Trace-sharing opt-in | High-priority follow-up: 36-file privacy change needs settings defaults and redaction audits |
| [#6939](https://github.com/tinyhumansai/openhuman/pull/6939) | Custom sub-agent allowlist | Defer: 37-file agent routing/authorization migration requiring role-policy E2E tests |

PRs #7722 (thread attachment placement), #7753 (cancel TinyBox shell), and #7721 (deadline bounded grounded close) were merged into upstream while this audit was in progress; their newer upstream commit history was subsequently merged into the integration branch. #7746 was already in the first upstream snapshot.

## Fork default-branch screening (sample, not exhaustive)

| Fork | Ahead / behind pinned upstream snapshot | Finding | Disposition |
|---|---|---|---|
| [antono4/openhuman](https://github.com/antono4/openhuman) | 911 ahead / 36,422 behind | Old divergent Indonesian README/automated timestamp workflows; no safe isolated feature identified | Exclude bulk merge |
| [leon-up9/openhuman](https://github.com/leon-up9/openhuman) | 5 ahead / 7,708 behind | Legacy Railway Dockerfile pinned to obsolete upstream revision; submodule checkout retry | Exclude incompatible pinned deployment |
| [yamilogic/openhuman](https://github.com/yamilogic/openhuman) | 1 ahead / 19,044 behind | Obfuscated JavaScript payload inside `app/postcss.config.js` | **Reject: untrusted supply-chain modification** |
| [Bhavesh1236/openhuman](https://github.com/Bhavesh1236/openhuman) | 1 ahead / 36,551 behind | Old `src/openhuman/todos` scheduler tests plus suspicious enormous PostCSS whitespace | Exclude stale/unverified patch |

Most additionally sampled default branches were behind upstream and had zero independent commits at the pinned comparison point. Some inaccessible repositories and default-branch mismatches returned 404; absence of results is **not** evidence that the fork is behind.

## Validation gates before merging

1. Run Rust formatting, `cargo check/test` for modified provider registry, security policy, web-chat cancellation, permission middleware, and task-sources domain.
2. Run frontend typecheck (`pnpm --filter openhuman-app compile`, `typecheck:e2e`), relevant Vitest, CI lane scope tests and asset checks.
3. Inspect Module bundle, CI Fast, PR Quality, Storage Mongo, and other platform checks on **the final PR head**; previous heads' green results do not validate later changes.
4. Resume and review the full ahead-fork census with a pinned upstream revision, triage each unique commit for value and harmful content, and record the decision before completing the user's exhaustive-audit request.
5. Keep `main` untouched and PR #5 draft until the above acceptance gates are met.

