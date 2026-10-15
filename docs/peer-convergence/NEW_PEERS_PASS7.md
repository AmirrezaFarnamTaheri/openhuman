# Peer convergence pass 7: 20 additional ZIP projects

**Status:** selective native implementation in draft PR #8; not exhaustive feature parity. All 20 archive central directories, nested folders and 10,750 eligible source files were inventoried. Static symbol extraction identified 82,739 candidate declarations. This is lexical evidence, **not** an assertion that every function was manually understood or ported.

## Architectural disposition by peer

| Peer | Native target | Why direct import is not appropriate |
| --- | --- | --- |
| 9router (newer ZIP) | Provider capability/fleet metadata and routing diagnostics | Existing Rust credentials, quota, role routing and network policy already own these |
| CLI-Anything | Permission-scoped typed app adapter catalog | Do not run a zoo of unmanaged CLI harnesses |
| Canny | Freshness-bound edit/test completion receipts | Existing TinyAgents ledger is authoritative |
| ECC | Skills/hooks quality governance | Existing skills, tool registry and policy own execution |
| Jev-Auto-Router | Typed model/effort candidate selection | No intercepting Codex proxy or silent billed-role overrides |
| JevRouter | Typed capability ranking with permission-before-decision | Existing JevRanker, MCP and approval gate |
| MineContext | Opt-in context-source capture and lifecycle | Existing privacy, memory, desktop stores |
| Paper2Slides | Provenance-backed slide storyboard and native render pipeline | Existing Rust presentation and artifact engines |
| SWE-agent | Coding benchmark/verification receipts and review phases | Existing TinyAgents execution, isolation and checkpoints |
| codegraph-rust | AST source graph, incremental invalidation, graph+vector retrieval | No duplicate SurrealDB/index server |
| graphify-dotnet | Graph exchange, confidence and visualization contracts | Keep Rust core; no embedded .NET stack |
| graphify-rs | AST parsers, EXTRACTED/INFERRED/AMBIGUOUS relations | Build on existing source evidence and memory |
| jev-codex-router | Shadow routing, effort/cost cockpit and fallbacks | Avoid traffic interception and new secrets |
| jev-model-router | Live verified model catalog and feasibility/price scores | Provider selection remains inside native factory |
| jev-research | Decision-model hypothesis/evaluation sources | Research sketches are not production APIs |
| jev-router-playground | Explainable router UI examples | No independent public relay |
| jev-rs | Typed local System-One provider adapter | Existing Jev clients/rankers are authoritative |
| mem0 | Scoped memory consolidation, deduplication and evaluation | Existing memory stores and deletion ownership |
| open-swe | Verified PR/CI lifecycle and check-run reconciliation | Existing TinyAgents and GitHub integration |
| ruflo | Bounded orchestration, budgets and graph/cost attribution | No second agent runtime/swarm database |

## Delivered in this branch

The opt-in Source Evidence tool has been extended with a conservative in-memory **source graph** operation. It resolves a small, explicit set of Rust and JavaScript/TypeScript relative module/import syntaxes only when exactly one authorized, observed file matches. Missing targets remain unresolved, multiply matching targets remain ambiguous, and all edges are labeled syntactic_only. Inbound impact is a **lower bound**, never an exhaustive call graph. No content or decision is persisted to a competing database.

The same filesystem tool supports deterministic lexicographic paging (after_path), maximum 32 eligible files/2 MiB per call and maximum 64 reported edges. Pages carry explicit partial flags and SHA-256 evidence. Read handles enforce the byte cap and Unix final-component no-follow behavior. Source snippets emitted to an agent can enter the active model's context; users must opt in and apply existing privacy controls.

## Higher-order convergence contracts

1. **Unified evidence fabric:** (workspace_scope, path, source_sha256, observed_generation, line_range, completeness) binds source graph, memory recall and tool outcomes; stale source invalidates derived references.
2. **Model-capability decision plane:** hard deterministic feasibility, permission, context and budget filters precede typed Jev choice; probabilities never grant permissions or silently move roles onto BYOK billing.
3. **Verified software factory:** existing checkpoints, file changes and tool results create an edit -> tests -> review -> PR -> latest-commit CI receipt chain. A model claim cannot substitute for a passing check after the latest edit.
4. **Context lifecycle:** proactive capture is explicit opt-in; memory facts preserve ownership, provenance, retention and deletion through graph retrieval.
5. **Artifact studio:** source-grounded storyboard and layout verification feed the existing presentation/document artifact engines and reusable media tools.
6. **Operator cockpit:** show model/account capability, latency/cost, source freshness, permissions, verification and unknown/partial states without fabricating green status.

## Acceptance gates not yet met

- Hosted Rust compilation and focused source_graph/source_evidence tests on the **latest exact PR head**, followed by rustfmt/clippy/full Rust suites, frontend/Tauri builds and E2E flows.
- Real provider authentication, quota, 429/error-scope and streaming differential smoke tests.
- Independent adversarial tests of Windows/macOS/Linux path semantics, symlink races, Unicode, huge directories, source pagination and graph correctness.
- Full AST symbol/call graph, incremental project-wide indexing, confidence calibration and memory integration remain unimplemented.
- Full Canny/SWE/Open SWE agent verification lifecycle, broader model-decision plane and artifact studio are **proposed, not shipped**.

**Licensing and provenance:** permission to examine or adapt peers does not rewrite each repository's actual license. Several uploaded archives state Apache, MIT or dual licensing; one lacks obvious root license indicators. Rewritten native concepts are preferred over copy/pasting peer subsystems. Preserve provenance and clear third-party notices before a release.

Full archive member census, source SHA-256/symbol inventory, method-name overlap and per-file preliminary triage are available in the separately delivered audit evidence bundle.
