# Peer convergence pass 12 — unified 36-archive source ledger

**Target:** draft PR #8. This is a reproducible structural audit and a native-integration decision map, **not** a claim that all upstream methods are semantically understood or ported.

## Complete archive input census

The ZIP inputs from all previous and latest peer cohorts were enumerated without executing archive code. After SHA-256 deduplication: **36 unique ZIP archives** (seven duplicate uploaded ZIPs removed); **28,232 central-directory members**, including **12,079 readable source candidates** and **8,457 candidate configuration/documentation entries**. The offline lexical/Python-AST inventory recorded **150,847 declaration rows** from approximately **113.9 MiB** source bytes.

Archive processing logged **49 issues**: 19 symlink/traversal-risk members excluded from direct source reads, 28 Python syntax/AST parse errors, and two oversized structured inputs. No cross-archive identical entire source-file SHA-256 group was found among indexed source files of at least 128 bytes. That does not rule out duplicated snippets, shared scaffolding, or semantically equivalent logic.

The machine-readable file/module/member inventory, per-file hashes, declaration locations, exception list, and the offline reproducibility scripts are delivered in the conversation's separately attached `OPENHUMAN_UNIFIED_PASS12_EVIDENCE.zip`. They are not embedded here because large generated indexes should not bloat the source repository.

**Source families audited:** 9router; CLI-Anything; Canny; Conduit; ECC; Jev-Auto-Router; JevLoop; JevRouter; MineContext; Paper2Slides; SWE-agent; Blink; CodeGraph Rust; Graphify .NET; Graphify Rust; Hono Jev Router; Jev-Codex Router; Jev; Jev Model Router; Jev Research; Jev Router Playground; Jev Rust; Jev Tree; Jevbus; Jevex; Mem0; Open SWE; Pi Jev Router; Ruflo; Skillbox; Stanley; Switchboard; Todo Jev; TypeSafe Jev Workflow; TypeSafe Skill Router; Winx Code Agent.

## Integration decisions by higher-order product plane

| Priority | Product capability | Authoritative OpenHuman owner | Acceptance standard |
| --- | --- | --- | --- |
| P0 | Source evidence and AST intelligence | `source_evidence`, host `SecurityPolicy`, memory index | Authorized, provenance-bound, incremental; no false dependency claims or unsafe path reads |
| P0 | Verified coding mission outcome | Native TinyAgents journal and GitHub check adapter | Observed post-edit tests, complete event chain, final exact commit SHA, clear partial/unknown states |
| P0 | Account-safe model fleet and Jev routing | One Rust provider factory/credential pool, Jev and Model Fleet | Deterministic hard permission, billing, context, quota and transport gates before probabilistic ranking |
| P1 | Memory and context convergence | Existing scoped memory store | Opt-in capture, source provenance, deletion, retention, contradiction and workspace boundaries |
| P1 | Skills and app adapters | Existing skill registry, preflight, workflows, tools, MCP permissions | Typed capability/permission contracts; no ungoverned second agent or CLI runtime |
| P1 | Artifact production | Existing document/presentation/rendering engines | Grounded storyboard, layout/accessibility QA, cancellation, reproducible output |
| P1 | Orchestration operator cockpit | Existing agent event stream and routing telemetry | Accurate live progress, budget, failures, stale/partial/unknown evidence, no fabricated health |
| P2 | Remote PTY and durable sessions | Desktop/TinyAgents sandbox, optional scoped adapters | Principal isolation, remote credential lifecycle, reconnect and termination safety |

A peer feature must be **reused**, **adapted**, **replaced only after a differential test**, **governed as an optional plugin**, or **explicitly rejected/deferred**. Structural similarity, same method names or permissive user instructions about licensing alone are not sufficient for direct copying. Actual licenses/notices remain relevant before distribution.

## This pass: concrete implementation and contract hardening

1. **Source graph comment safety:** a JavaScript `export { value }; // from './fake'` could be misinterpreted as a real dependency because only the statement's first token was checked against lexical comments. The parsed `from` token itself must now be visible code; real imports whose destination names contain `from` remain allowed. Focused Rust regression tests were committed.
2. **Provider submit guard:** an API handler invoked outside the CloudProviderEditor form could previously accept a credential-bearing off-loopback plaintext HTTP route or a remote plaintext image override. The handler now applies the same HTTPS/loopback transport rule before **any** credential probe, configuration persistence, or key storage. No-key LAN model endpoints remain supported. React/Vitest regression cases were committed.
3. **Local-check receipt integrity:** the journal projection now requires internally consistent `status`, `passed`, `exit_status`, `scope`, and `task_verified=false` fields before recognizing a local check. Contradictory or incomplete metadata cannot fabricate positive verification. Regression tests cover malformed facts.

## Mandatory release gates

- Cargo/Rust toolchain was not present in this execution container, and direct external clone/install was blocked by DNS. These Rust source edits are **not compile-verified**. React/Vitest is likewise not run against the complete OpenHuman checkout here.
- All focused tests, full workspace Cargo check/test/fmt/clippy, frontend TypeScript/Vitest and builds, Tauri, platform/E2E checks, controlled provider transport tests, and GitHub Actions **on the latest exact PR head** are still needed.
- The larger P0/P1 planes above remain partially implemented or proposed; this document does not mark them complete. Keep PR draft and do not merge unverified changes.

See `SOURCE_EVIDENCE.md`, `VERIFICATION_RECEIPTS.md`, `PASS8_EVIDENCE_AND_VERIFICATION.md`, and `PASS10_NEW17_AND_SECOND_ORDER.md` for the preceding cohort findings.
