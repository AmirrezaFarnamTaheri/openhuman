# Absorption review: science, continuity, visualization and reusable code

Date: 2026-10-18. This sidequest accompanies [PR #14](https://github.com/AmirrezaFarnamTaheri/openhuman/pull/14). It covers every explicitly named repository, both organization inventories and the requested boilerplate search as a bounded discovery sample. Twenty repository snapshots received selected source-level review; this is not a claim to have audited every file or reproduced peer tests and benchmarks. No peer implementation was installed or imported.

## Recommended direction

The highest-value additions improve what OpenHuman can retain, inspect and verify: faithful session imports, evidence packets that explain omissions, revision-linked provenance, resource-aware workload planning, and configurable reusable-code inspection. The smallest useful changes include collision-safe content identities, safe Markdown fences, subscription cleanup, accessibility descriptions and honest unknown states. The larger opportunities are a transcript-import module, a research workflow pack and a reviewer-export module. They fit existing owners; a wholesale replacement runtime does not follow from this review.

The immediate implementation remains the active TinyTools/TinyAgents/OpenHuman contribution. It establishes bounded structural edits, actual-execution observation and host policy boundaries that several proposed additions would consume. Completion of that work and its existing gates precedes any expansion proposed here.

### Reading map

- [Science, K-Dense and Multica](2026-10-18-science-and-platforms.md): 14 concrete candidates, organization inventories, source/license caveats and host mappings.
- [Continuity, trajectories, pstack and scientific profiles](2026-10-18-continuity-and-verification.md): memory import, evidence allocation, verification records and optional content.
- [Boilerplate repositories and search sample](2026-10-18-boilerplates.md): generators, parameter forms, bounded variant tests, desktop smoke and service patterns.
- Victory and actix-net are reviewed below, with their own pinned sources and current OpenHuman overlap.

## Disposition of every requested source

| Source | Best unit to absorb | Destination and initial decision |
| --- | --- | --- |
| K-Dense-AI organization | BYOK's observed/inferred provenance and reviewer exports; scientific-agent-skills' effective resource planner and bounded dataset profile | Selected host/artifact and optional analysis capabilities; inspect each file and dependency separately |
| science-superpowers | Preregistration audit fields, research-specific validation guidance and data-hash checks | Optional research workflow; a read-only audit should precede any mutating freeze operation |
| claude-scientific-writer | Document-neutral artifact inventory and narrow metadata parsing cases | Artifact/document module; exclude the separately restricted bundled docx subtree |
| scientific-agents | A few selected profiles, content-size checks and generated catalog parity | On-demand TinySkills content; preserve current application and project instruction authority |
| agentic-data-scientist | Generation-scoped review state, explicit terminal status and agent-local loop regression cases | Existing TinyAgents lifecycle and host presentation; its ADK controller is not a direct fit |
| multica-ai organization | Canonical multi-file identity and archive collision test questions | Design references for TinySkills; direct source reuse needs separate license clearance |
| pstack | Typed PR snapshots, evidence records and fake-clock queue tests | Connector-backed host tooling; rewrite missing/stale-check readiness semantics |
| ContextLattice | Typed selected/omitted evidence packet and deterministic fusion | TinyMemory contracts and TinyAgents context preparation; optional engine adapter only after conformance work |
| Victory | Accessible chart container, controlled brush/zoom, bounded visible-domain rendering and coordinated interaction patterns | Current React dashboards; consider a scoped dependency only for a concrete analytical view |
| trajectory | Normalization types, durable source identity, tool/result linkage and partial-import diagnostics | Highest-priority complete module candidate in tinymemory-integrations |
| actix-net | Capacity accounting, readiness, drain-state reporting and loopback test fixtures | Existing RPC/TinyBus/Tokio owners; no runtime migration proposed |
| Boilerplate search page 2 | A reproducible candidate list | Discovery only; exact browser page membership was not reproduced and sampled repositories were not fully audited |
| react-boilerplate | Paired source/test generation and variant linting | Future TinyTools multi-file patterns, modernized for current React/TypeScript |
| electron-react-boilerplate | Renderer-to-host smoke assertions and explicit subscription cleanup | Existing Tauri tests and host-event adapters |
| ChristianLempa/boilerplates | Typed/grouped parameters, bounded combination matrices and explicit secret metadata | Chunk inspection UI and future versioned schema extensions; do not bulk-import its legacy library |
| aspnetcorehero/Boilerplate | Permission/audit display and application/domain/infrastructure boundaries | UI design reference and selected future service templates; no C# runtime adoption |

Each decision above is an engineering recommendation. The linked reviews distinguish directly read implementation from README claims, discovery-only entries and future acceptance work.

## Victory: composable analytical interactions

Inspected commit: [`75549cf8c67e3ea220f862eb7f530c36d540d41c`](https://github.com/FormidableLabs/victory/commit/75549cf8c67e3ea220f862eb7f530c36d540d41c). The inspected package declares version 37.3.6, a React peer requirement of `>=16.6.0`, and separately packaged charts, containers and shared events. GitHub's license metadata is inconclusive, but the actual [LICENSE.txt](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/LICENSE.txt) is MIT and includes contributor notices. [Package manifest](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory/package.json)

### Existing overlap

OpenHuman already uses Recharts `^2.15.0`; the cost and token dashboards share a tooltip component, localized labels and explicit empty-domain handling. Their unit tests mock chart primitives because DOM-only tests lack layout measurement. This supports a focused interaction/accessibility improvement and a real-browser check, not a claim that the application lacks charts. [Current package](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/app/package.json), [token chart](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/app/src/components/dashboard/TokenUsageChart.tsx), [chart tests](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/app/src/components/dashboard/TokenUsageChart.test.tsx)

| Scale | Candidate | Adaptation and useful acceptance evidence | Estimated effort |
| --- | --- | --- | --- |
| Tiny detail | Stable chart title/description IDs and ARIA linkage | Add an accessible summary and data-table alternative to existing dashboards; test multiple charts, empty data and keyboard/screen-reader names | 0.5–1.5 days |
| Primitive | Controlled visible-domain state and bounded data selection | Keep filtering, zoom state and the displayed aggregate tied to one data revision; preserve units, timestamps and missing samples | 2–4 days |
| Widget | Linked usage/cost brush with a nearest-point tooltip | Compare current Recharts capabilities before adding a dependency; test pointer, touch, resize, keyboard alternatives and synchronization | 3–5 days |
| Feature | Research result view with scatter/error bars, distributions and selected observations | Use a typed data/units/provenance contract and explicit uncertainty; test real rendering and export | 1–2 weeks |
| Module | A scoped Victory-based analytical viewer | Measure bundle, render and interaction cost on representative data, isolate the dependency to that view and preserve current dashboard behavior | 2–4 weeks |

These are contributor estimates including focused tests, not measured delivery promises.

### What the source contributes

The [base container](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-core/src/victory-container/victory-container.tsx) connects optional titles/descriptions to generated IDs, ARIA attributes and an SVG role. That is a concrete accessibility pattern. A chart's labels alone do not make every visual interaction keyboard-accessible; test the actual product workflow.

The [zoom container](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-zoom-container/src/victory-zoom-container.tsx) can select the visible domain and downsample it; the default target when downsampling is explicitly enabled is 150 points. The [zoom helpers](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-zoom-container/src/zoom-helpers.ts) bound zoom around minimum domains and throttle interaction handlers. These are useful design references, but downsampling can conceal narrow peaks or uncommon observations. A scientific view must disclose decimation and retain access to the underlying records; a point count is not a semantic error bound.

The [container composer](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-create-container/src/create-container.tsx) combines zoom, selection, brush, cursor and Voronoi behavior through hooks and event mutations. Its event-merging helper explicitly does not handle `eventKey` or `childName`. That limitation matters for complex linked dashboards: adopt a narrowly tested composition rather than assuming arbitrary containers combine correctly.

No Victory runtime or performance benchmark was executed during this audit. The package metadata's React range also does not prove every interaction works under OpenHuman's React version. Preserve MIT notices for copied or substantially adapted implementation; review the selected package's dependency licenses if using it directly.

## actix-net: readiness, accounting and shutdown semantics

Inspected commit: [`acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941`](https://github.com/actix/actix-net/commit/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941). The workspace declares Rust 1.89 and `MIT OR Apache-2.0`; per-crate license files are present. The inspected [workspace manifest](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/Cargo.toml) and [MIT license](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/LICENSE-MIT) are the evidence, rather than treating repository metadata as a complete license inventory.

### Existing overlap

OpenHuman's RPC server already uses Axum and Tokio, accepts an explicit cancellation token, performs graceful shutdown, and runs exit cleanup even after a serving error. Its classification layer already distinguishes backpressure. TinyAgents has bounded dispatch and cancellation. A new Actix runtime would overlap with these owners; the useful port is a specific state machine or regression that demonstrates a current gap. [RPC serving path](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/crates/openhuman-rpc/src/server/serve.rs), [classification](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/crates/openhuman-rpc/src/server/classify.rs)

| Scale | Candidate | Owner and adaptation | Estimated effort |
| --- | --- | --- | --- |
| Primitive | Admission capacity increments before a connection becomes visible; release only once | Existing connection/run budgets, with drop/cancel/race regressions | 1–3 days |
| Primitive | Explicit readiness separate from request execution | Current Tower/Tokio admission interfaces; preserve overload errors and avoid an unnecessary trait bridge | 2–4 days if a gap is reproduced |
| Feature | Drain outcome distinguishes graceful completion from forced timeout | RPC/TinyBus lifecycle, with in-flight requests, new-admission refusal and cleanup assertions | 2–4 days |
| Test utility | OS-assigned loopback port with retained listener and explicit server cleanup | Existing native test harness; avoid free-port probe then rebind races | 0.5–1.5 days |
| Whole crate | `actix-codec` or `actix-service` in a genuinely Actix-native module | Evaluate direct interoperability, MSRV, dependencies and thread model first | 1–2 weeks for a narrow adapter; no adoption currently recommended |
| Whole plane | Replace the RPC/transport runtime | No demonstrated product benefit in this audit; substantial ownership and behavior migration | Not recommended |

The [service trait](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-service/src/lib.rs) uses shared-reference `poll_ready` and `call`, supports `no_std` with allocation and has separate service factories. Readiness is documented as best effort: a subsequent call can still fail, and a caller may invoke `call` without polling readiness. An adapter must preserve those semantics. It must also respect that Actix commonly permits local, non-`Send` futures; these are not automatically interchangeable with OpenHuman's cross-thread futures.

The [worker implementation](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/worker.rs) documents the connection counter's ownership and makes the count visible before publishing a connection. It separates unavailable, available and shutdown states, and reports whether shutdown completed gracefully or required a timeout. These are concrete invariants to compare with current budgets and shutdown receipts. Copying the server wholesale is unnecessary to use the test questions.

The [test server](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/test_server.rs) binds an OS-assigned loopback port and retains its server handle. The [TLS feature matrix](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-tls/Cargo.toml) supports several TLS generations; importing all of it would broaden the dependency graph. Select only the implementation actually needed, and retain the host's current certificate and credential policy.

## Suggested implementation sequence

1. Finish the active structural-code, permission and actual-execution contributions. Their owner landing, exact-head CI and host composition gates remain authoritative.
2. Prototype two transcript codecs in TinyMemory. Require stable full/chunked identity, duplicate-safe replay, tool/result linkage, loss accounting and namespace enforcement.
3. Add selected/omitted evidence receipts around existing recall. Explicit zero budget stays zero; final serialization must fit or refuse; approximate counts stay labeled approximate.
4. Extend provenance from trusted write receipts and observed process results. Keep inferred shell effects and caller-supplied claims distinguishable. Add a reviewer export only after source identity and omissions are specified.
5. Add a small chunk-parameter inspector and bounded generation matrix. Treat multi-file publication, secret references and dependency installation as separate contracts.
6. Offer optional research profiles and a bounded dataset inspector. Keep scientific interpretation and preregistration claims tied to actual evidence.
7. Add chart and network refinements only when product measurements or a reproduced defect justify them.

The detailed reviews include larger module alternatives so they remain assessable. Priority here reflects fit and demonstrable gaps, not repository popularity. A proposal becomes a port only after its owner, source/notice record, input/output contract, migration implications and behavioral acceptance evidence are concrete.
