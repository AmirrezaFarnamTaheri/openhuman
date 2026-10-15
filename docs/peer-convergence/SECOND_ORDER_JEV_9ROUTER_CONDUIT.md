# Second-order peer convergence: Jev, 9router, Conduit

> 2026-10-15. Evidence: all three supplied ZIP central directories and the previously supplied sixteen peer archives were enumerated; static source signatures and routes were indexed. Enumeration is not equivalent to executable or line-by-line proof of parity.

## Archive size and product modules

| Peer | ZIP members | Extracted source/document candidates | Static declarations/routes | Main subsystems |
| --- | ---: | ---: | ---: | --- |
| Jev | 348 | 145 | 708 | `internal/{scan,run,baseline,calibration,evidence,typesafe,usage,cost}`, CLI hook |
| 9router | 1,630 | 1,427 | 3,312 | `open-sse/{services,providers,translator,handlers}`, `src/lib/{oauth,db,modelCatalog,tunnel,network}`, provider UIs |
| Conduit | 1,275 | 998 | 11,453 | `crates/{source/graph,state/scope,runtime/authz,transport/mcp,change/edit,rtk,zap,ripgrep}` |

These counts include duplicated nested trees and vendored sources. Files were enumerated, but behavior was not manually checked in every line. Historical Git objects, generated assets and executables are not independent product modules.

## Jev → OpenHuman: bounded source intelligence

High-value behaviors: declaration-first semantic discovery, narrowing long reads to evidence windows, bounded requests/deadlines, snapshot-bound verification, calibration ledgers and source-anchored results. Jev is an external Claude Code hook: its API key, execution environment and source-context export must **not** be silently copied into OpenHuman. Adapt a new *opt-in, read-only* native source-intelligence plane and route retrieval through OpenHuman's tool, approval and MCP ownership. First prove root/symlink containment, immutable snapshot generation, output/token budget, source-hash drift, and scoped access in tests; then add consented model relevance ranking.

## 9router → OpenHuman: provider capabilities, quotas and evidence

High-value behaviors: per-account model lock and quota resets, provider capability metadata, provider-model negotiation, error classification, fallback selection, account refresh, and usage surfaces. Existing OpenHuman `inference/provider/factory/key_pool.rs` already owns round-robin/least-busy/weighted selection, sticky sessions, lease counting, failover, cooldown and progressive 429 backoff. Recent work on this PR also added the local 9Router-compatible endpoint and eight key-based provider presets. Do not transplant the Next.js proxy server or duplicate the credential database.

Important correctness boundaries: an invalid request must not mark a healthy key as rate-limited; authentication must be proven by an authenticated operation rather than public `/models` reachability; a model-specific failure need not revoke the entire provider; keep explicit per-role billing routes and no-redirect credentialed egress. Add tests for 400/401/403/404/429/5xx distinctions, Retry-After precedence, account-level vs model-level availability, and multi-account refresh.

## Conduit → OpenHuman: derived graph, optimistic edits and bounded context

High-value behaviors: source anchor records, conservative dependency/call edges with `resolved/ambiguous/unresolved` and confidence, rebuildable graph projections, graph completeness/staleness, scoped receipts, hash-anchored edits, and output compression budgets. The supplied source graph is a *derived projection*, not a new authoritative file database. Port that invariant: keep OpenHuman's existing workspace, security policy and MCP server authoritative. Reject edits when a read-time digest has drifted; require existing host approval before mutations; do not equate static call edges with runtime truth.

Conduit also includes sizeable full ripgrep, RTK, and Zap trees. Integrating all three into OpenHuman would be an architectural regression without distinct needs and benchmarks; selectively absorb source-graph/query interfaces, formatting contracts and command-output limits.

## Second-order synthesis

1. **Evidence plane:** unified immutable source references `{root_scope,source_hash,generation,path,line_range,completeness}` from code, tools, and memory.
2. **Execution plane:** preserve native TinyFlows/TinyAgents checkpointing, authorized tools and replay outcomes; avoid a second job or idempotency ledger.
3. **Routing plane:** one source of provider capability, auth health, rate limits and optional fallbacks; never silently inherit another role's billable provider.
4. **Retrieval plane:** combine source graph and code queries with scoped TinyMemory recall; distinguish observation, deduction and model relevance.
5. **UI plane:** expose freshness, model availability and evidence/permission status in existing panels; no frontend-only feature pretending to have a backend.

## Independent implementation gates

- Native source catalogue + immutable snapshot hash + path-bound evidence, including permissions and drift tests.
- Bounded semantic retrieval + reproducible calibration and request budgets; no remote file upload without opt-in.
- Source graph projection + conservative edge semantics + invalidation and impact queries.
- Model-level quota/fallback across existing Rust fleet, with user-visible explanation and precise error scopes.
- Optimistic multi-file edit transaction with existing approval, audit and recovery receipt.
- Integrate evidence, identity, routing and trace semantics into existing MCP/agent UI surfaces.
- Latest exact-head Rust/Tauri/frontend/storage/format/security and external service smoke tests before merge.

## Shipping and licensing

Archive claims were not verified as production benchmarks; model price/latency, full source search accuracy, OAuth capabilities and graph soundness need independent tests. Do not equate a ZIP label or request to assume MIT with proof of license: the supplied Conduit tree contains Apache-2.0 components (`rtk`/`zap`) and public-domain ripgrep files. Preserve provenance and obtain licensing clearance before copying substantial implementation. PR #8 remains draft until validation succeeds on its exact head.

Full file-by-file archive path and source-symbol inventories are available in the accompanying `convergence_v3_evidence.zip` audit artifact.
