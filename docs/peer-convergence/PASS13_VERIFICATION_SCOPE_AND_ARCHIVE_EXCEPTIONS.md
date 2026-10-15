# Pass 13 — fail-closed software-check evidence and archive exception triage

**Status:** committed on draft PR #8; **not Rust-compiled or CI-verified**.

## P0 correctness: unrelated successful checks cannot erase earlier failures

Before this change the local-check journal could report `local_check_observed` when
a failing Cargo test was followed by a successful formatter or build. The
projection tracked last-success/last-failure offsets, not what was actually
checked. It also treated tool names it did not recognize as non-mutating:
a newly added plugin or MCP editor could write files after a green test
without invalidating the local-check freshness result.

The ShellTool now attaches a **fixed, privacy-safe check category**
(`check_kind`, such as `cargo_test` or `cargo_fmt`) to its existing host-only
TinyAgents result metadata. It does not persist shell text, arguments,
environment, source paths, stdout, stderr, or model/user input in that receipt.
The read-only journal projection validates the category against a constant
allowlist and exposes `unresolved_check_kinds` and
`unattributed_failures` in its existing `run_local_checks` response.

A check category is *not* an invocation identity. A successful
`cargo test -p frontend` does not prove that a previous failed
`cargo test -p backend` was rerun. All failures therefore remain unresolved
for that durable run; a **separate clean run** can provide new successful
evidence after correction. Old category-less receipts remain readable, but
their failures are conservatively unattributed. The `task_verified=false`
field is never promoted by a local check.

Unknown tools now count as potentially source-mutating and invalidate earlier
freshness evidence. Only a small, explicitly known set of native read-only
tool names avoids invalidation. This conservative choice may show stale for
harmless unknown tools; until the harness records cryptographically grounded
effect/permission metadata, falsely treating unknown code writers as read-only
would be worse.

Focused Rust regression cases cover differing check types, same-kind commands
without invocation proof, legacy failures, unknown plugin writes, known read
operations, malformed categories and private-argument redaction.

## P0 privacy: source-evidence boundary

`source_evidence` already excludes common cloud key stores and .env files.
This pass additionally denies `.openhuman` workspace stores,
`.netrc`, `.envrc`, `.git-credentials`, `token.json`, `auth.json`
and `service-account.json`, both for directory walks and explicitly
addressed files. It deliberately continues to index legitimate code such as
`src/auth.ts`. The opt-in tool does not independently send network requests,
but its outputs may enter the configured main model's context.

## Full 36-archive ledger: reproducible exceptions

The conversation-delivered pass-13 audit re-enumerated 40 input ZIPs, 36
unique after SHA-256 deduplication, and 28,232 ZIP members. Under its explicit
filename and vendor eligibility rules, it indexed 20,386 readable source/
config/document records and recorded 51 exceptions. Counts differ slightly
from pass 12 where the eligible extensions/vendor rules differed.

- 19 ZIP symlinks or path-risk members were skipped rather than followed.
- 27 **production/automation** Python modules in the uploaded Open SWE ZIP
  contain invalid Python-3 multiple-exception syntax, for example
  `agent/analytics/usage.py:39`: `except OverflowError, OSError, ValueError:`.
  Correct to parenthesized exception tuples, then differential-test actual
  exception handling before adopting any implementation.
- One SWE-agent historic Python fixture contains Python-2 print syntax.
- Four oversized tokenizer/router/lock structured inputs were recorded,
  not automatically interpreted, imported or executed.

The detailed per-file SHA-256 inventory, module-family signals, exceptions CSV
and offline reproducibility script are available in the separate conversation
artifact `PASS13_UNIFIED_AUDIT.md` / pass-13 evidence bundle.

## Higher-order convergence remains bounded

Do not replace OpenHuman's single Rust provider factory, security policy,
TinyAgents journal, memory owner, skill/runtime owner or artifact renderers
with independent peer services solely on name similarity. Candidate primitives
from CodeGraph/Graphify/Jev, Canny/Open SWE/SWE-agent, 9router/JevLoop,
MineContext/Mem0, CLI-Anything/Skillbox and Paper2Slides still require
source-generation provenance, hard permission and billing feasibility,
lifecycle ownership and true before/after behavioral tests.

**Release gates not met:** focused Cargo tests for local_checks,
verification_tests and source_evidence, full fmt/check/test/clippy, frontend
tests and Tauri/E2E, cross-platform security tests, live disposable-provider
routing, plus CI against the final exact PR head. This environment lacks
Rust/Cargo and outbound GitHub DNS; GitHub Actions have not reported exact-head
runs from connector-created commits. Keep PR draft until these checks pass.

Original third-party licenses and notices remain relevant; an instruction to
assume MIT is not permission to mislabel or redistribute another license.
