# Remaining OpenHuman work

This is a working status, not a completion report. Changes are published in
[the personal fork PR #17](https://github.com/AmirrezaFarnamTaheri/openhuman/pull/17).
The installed OpenHuman application has not been replaced.

## Current local progress

- Conversation categories/project groups, archive/restore, collapse preferences
  and nested child disclosures are implemented locally. Pin/category/archive
  writes share a per-thread queue. Focused frontend regressions pass.
- Tool and skill actions have flatter inputs, preserved search line breaks,
  readable diagnostics and no fabricated retry/attempt controls. Focused checks pass.
- NVIDIA catalogue success can remain explicitly unverified rather than blocking
  credential saving when authentication cannot be distinguished from public access.
  Explicit rejection still fails. Focused API/provider checks pass; live inference
  has not been established by these mocks.
- Always-allowed tool additions and shell-window preference have local UI/API
  changes. Default overall wall-clock ceiling is disabled; explicit overrides and
  individual-call safeguards remain. Per-conversation shared budgets are pending.
- Runtime cache fingerprint covers persisted settings and explicitly skipped
  runtime route/endpoint/directory inputs. Frozen prompt/tool generation refresh
  is still pending. Auth-profile write attempts now invalidate a revision scoped
  to the backend scope or file directory, including partial keychain failures.
  New credential invalidation tests pass; external-process writes
  are outside this process-local tracker.
- Typed transient pre-content provider failures can rotate keys locally; partial
  answers are not replayed. Eleven focused credential-fleet tests pass.
- TinyAgents queue mutation draft previously passed 25 focused tests. New durable
  inbox admission passed three focused tests. Broader schema tests found a
  declaration issue, now corrected and verified by all 83 ledger-driver tests. Admission is not
  recipient delivery and does not implement Continue.

## Required work before completion

- Finish durable recipient admission/consumption, idempotent delivery receipts,
  scoped direct child messaging and execution-backed continuation/tree controls.
- Finish shared persisted time/token reservations and model-visible remaining
  budgets across parallel descendants and restart.
- Submit owner changes upstream with explicit authorization; consume only once
  available, respecting repository ownership rules.
- Implement the approved right run panel against those capabilities. Its approved
  design and implementation plan do not establish a functioning panel.
- Wire queue edit/reorder/send-now controls to available owner contracts.
- Complete existing-chat prompt/tool successor-generation application and verify
  secret-store-only credential changes refresh cached clients.
- Complete MCP/tool credential failover in its owning component and adapter.
- Complete durable/sidecar-only delegated attachment previews.
- Finish focused Rust verification, architecture/CI gates and actual UI evidence.
  A previous full Rust suite stalled and is not a passing result.
- Resolve any external release/registry permission failures through the owning
  account; no external resources, credentials, pushes or releases changed here.

## Publication and latest verification

- The user's OpenHuman fork PR #17 is open. Published head `deeaec453d` includes
  CI compilation fixes, current release expectations, runtime test isolation,
  asynchronous workflow import readiness, current live-voice RPC coverage, and
  preservation of MCP sandbox grants. CI run `39979564208` and Rust E2E run
  `39979564425` are still running; no all-green claim is supported.
- Local commit `76ef8501ee` adds approval fixture diagnostics and a reachable
  release-security reference. It is waiting for the current CI run to finish
  before the next push, avoiding cancellation of unfinished verification.
- The full documentation generator initially failed with disk error 112. A
  source-backed candidate was regenerated with the existing capability snapshot,
  correcting the default autonomy flag and the removed structural-code feature.
  Snapshot checking passes; compiled capability validation is still required.
- Cargo's package-specific cleanup (`cargo clean --package openhuman --profile
  dev`) subsequently removed 11.1 GiB of generated build files. Local Rust
  test-target checking has resumed; no user cleanup is needed for this blocker.
- The fork CI image build `39954844471` succeeded and published to
  `ghcr.io/amirrezafarnamtaheri/openhuman_ci`. A desktop release has not been
  published. The approved run-panel scope remains incomplete.
- Published the explicitly approved TinyAgents dependency draft PR:
  https://github.com/tinyhumansai/tinyagents/pull/567. Its current-upstream branch
  passed 72 ledger tests, 26 queue tests, formatting and scoped Clippy. All
  remote CI checks passed; PR 567 is now ready for review. Upstream availability
  is still required before host consumption.
- 292 focused frontend tests, 83 owner ledger-driver tests, 25 owner queue tests
  and 50 audit-script tests passed. These do not establish full completion.
- Fresh Rust layout, crate-chain, hosted-code boundary, feature-forwarding,
  storage-owner and storage-bypass gates pass. Changed frontend formatting passes.
- 69 focused root Rust tests passed, including publication safety. An expanded
  registry suite found one stale product-feature contract (54 passed, 1 failed);
  source correction is awaiting rebuilt tests.
- Latest focused frontend verification is 313 passes after scoped dependency
  security updates; typecheck, production build and new-test lint pass.
  Production dependency audit is clean; development-tooling audit has residual
  advisories. Four credential fingerprint and three revision tests pass.
  The registry diagnostic identified three missing hosted media names; they
  have been added to the fixture without removing gated product tools. All 32
  frontend presentation tests pass; rebuilt root catalogue verification is pending.
  Core Rust formatting passes.
  The docs generator needs Git Bash on PATH and must wait for Cargo to be idle.

## Latest CI repairs in progress

- Fixed Windows path normalization in the runtime-boundary scanner. Six focused
  scanner tests pass. Migrated three existing flow baseline records to their
  current paths/import and removed two vanished records; the gate passes with
  81 exact entries, without growing its allowances.
- Process-static ownership and SaaS ambient-state gates pass. Artifact writers
  now carry explicit dispatch context into their blocking publication task.
  Added a context-isolation test; its root build is still running.
- Source review identified a remaining artifact root/ancestor opening race.
  The new directory-handle walk rejects links at every canonical path component;
  Unix link and Windows junction regression coverage is awaiting compilation.
  The scoped storage writer replacement remains unchanged.
- Fixed the module-source gate's false positive on the boundary auditor's exact
  read-only Cargo tree inspection. Tests still reject build substitution,
  additional same-line activation and activation outside the auditor. Eight
  focused tests pass.
- Regenerated the frontend provider-chain block from App.tsx and repaired its
  Windows path assertion. Route checks now accept the existing static imports
  and still reject dynamic lazy paths or a split without a loading boundary.
  Twelve focused documentation/route tests pass.
- The broad script suite is not green: the first run had 756 passes, 174 failures
  and 29 skipped tests. Many failures require a POSIX shell/fixture environment;
  a rerun with Git Bash found a shell fixture hanging and was terminated with
  its own process tree. Do not present either run as passing CI.

## Review response and verified repairs (latest)

- TinyAgents PR 567 is updated to e575352d1. Addressed the exclusive inbox paging
  review, added a 503-admission regression, and documented the cursor contract.
  All 117 local ledger tests, scoped all-targets Clippy, formatting, and all remote
  CI checks pass. The review thread is resolved and the PR has no merge conflict.
- TinyTools PR 74 is updated to 8ac07cc after merging upstream main and resolving
  its optional code-crate version conflict. Local all-features tests/doctests,
  all-targets build, Clippy and formatting pass. New fork/upstream CI is pending.
- TinyAgents fork PR 1 has no unresolved inline comments or merge conflicts;
  its recorded head has passing CI. An inactive paid review subscription is an
  external service state, not a source-code finding; no billing was changed.
- Latest artifact verification is 138 passing tests, including Windows junction
  swaps, migration writes, and explicit dispatch-context isolation. The rebuilt
  Rust tool-catalogue fixture test also passes.
- Native Linux script validation initially had 941 passes and 14 failures.
  The relevant workflow, coverage, module-pin, module-build and i18n fixes pass
  111 focused tests; latest coverage pipeline tests pass 29. A fresh full native
  run still needs an isolated Linux Rust installation for the real bootstrap
  test. Initial installation hit a network reset; retry setup is in progress.
- TinyChannels was repinned to published v0.1.38 with official checksums and
  unchanged bus source. Module-pin and monotonicity gates pass. Both manifests
  resolve; root online metadata fetched missing Windows dependency packages.
- Repaired the CI image workflow to publish in the current fork owner namespace.
  The failed fork run attempted to publish to the upstream registry namespace.
  A focused namespace regression passes. Publication itself is not yet verified.
- The user now authorizes fork build, release and publication after validation.
  No OpenHuman PR, push or new release has been created at this point. The
  remaining run-panel implementation list above is still outstanding.

- Final owner review update: TinyTools PR 74 at a49d5f9 has all fork and upstream CI jobs passing, including 90 percent per-file coverage. Both upstream PRs are conflict-free.
- Full native Linux scripts: 962 pass, 5 skip, 0 fail (967 total). Typecheck, i18n coverage, English checks, and production dependency audit pass. New archive-to-create regression: all 17 thread-list tests pass.
