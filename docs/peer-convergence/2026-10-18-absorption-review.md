# Absorption review: science, continuity, visualization and reusable code

Date: 2026-10-18. This sidequest accompanies [PR #14](https://github.com/AmirrezaFarnamTaheri/openhuman/pull/14). It covers every explicitly named repository, both organization inventories and the requested boilerplate search as a bounded sample followed by selected source and license inspection. Thirty distinct repository snapshots received selected source-level review; this is not a claim to have audited every file or reproduced peer tests and benchmarks. No peer implementation was installed or imported.

## Recommended direction

The highest-value additions improve what OpenHuman can retain, inspect and verify: faithful session imports, evidence packets that explain omissions, revision-linked provenance, resource-aware workload planning, and configurable reusable-code inspection. The smallest useful changes include collision-safe content identities, safe Markdown fences, subscription cleanup, accessibility descriptions and honest unknown states. The larger opportunities are a transcript-import module, a research workflow pack and a reviewer-export module. They fit existing owners; a wholesale replacement runtime does not follow from this review.

The TinyTools and TinyAgents owner contributions now establish bounded structural edits, actual-execution observation and host policy boundaries that several proposed additions would consume. Their exact-source owner matrices passed. OpenHuman's assembled contribution is published at [462cbfe8](https://github.com/AmirrezaFarnamTaheri/openhuman/commit/462cbfe8df4b496bd69ca0d2d55d5d7fed7b5e1d), with current host CI and combined native targets still pending at this refresh. The proposals below remain investigation outcomes; they are not additional implementation claims.

### Reading map

- [Science, K-Dense and Multica](2026-10-18-science-and-platforms.md): 15 concrete candidates, organization inventories, source/license caveats and host mappings.
- [Continuity, trajectories, pstack and scientific profiles](2026-10-18-continuity-and-verification.md): memory import, evidence allocation, verification records and optional content.
- [Boilerplate repositories and search sample](2026-10-18-boilerplates.md): generators, parameter forms, bounded variant tests, desktop smoke and service patterns.
- Victory and actix-net are reviewed below, with their own pinned sources and current OpenHuman overlap.

## Disposition of every requested source

| Source                      | Best unit to absorb                                                                                                                       | Destination and initial decision                                                                                                                        |
| --------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| K-Dense-AI organization     | BYOK's observed/inferred provenance and reviewer exports; scientific-agent-skills' effective resource planner and bounded dataset profile | Selected host/artifact and optional analysis capabilities; inspect each file and dependency separately                                                  |
| science-superpowers         | Preregistration audit fields, research-specific validation guidance and data-hash checks                                                  | Optional research workflow; a read-only audit should precede any mutating freeze operation                                                              |
| claude-scientific-writer    | Document-neutral artifact inventory and narrow metadata parsing cases                                                                     | Artifact/document module; exclude the separately restricted bundled docx subtree                                                                        |
| scientific-agents           | A few selected profiles, content-size checks and generated catalog parity                                                                 | On-demand TinySkills content; preserve current application and project instruction authority                                                            |
| agentic-data-scientist      | Generation-scoped review state, explicit terminal status and agent-local loop regression cases                                            | Existing TinyAgents lifecycle and host presentation; its ADK controller is not a direct fit                                                             |
| multica-ai organization     | Canonical multi-file identity and archive collision test questions                                                                        | Design references for TinySkills; direct source reuse needs separate license clearance                                                                  |
| pstack                      | Typed PR snapshots, evidence records and fake-clock queue tests                                                                           | Connector-backed host tooling; rewrite missing/stale-check readiness semantics                                                                          |
| ContextLattice              | Typed selected/omitted evidence packet and deterministic fusion                                                                           | TinyMemory contracts and TinyAgents context preparation; optional engine adapter only after conformance work                                            |
| Victory                     | Accessible chart container, controlled brush/zoom, bounded visible-domain rendering and coordinated interaction patterns                  | Current React dashboards; consider a scoped dependency only for a concrete analytical view                                                              |
| trajectory                  | Normalization types, durable source identity, tool/result linkage and partial-import diagnostics                                          | Highest-priority complete module candidate in tinymemory-integrations                                                                                   |
| actix-net                   | Capacity accounting, readiness, drain-state reporting and loopback test fixtures                                                          | Existing RPC/TinyBus/Tokio owners; no runtime migration proposed                                                                                        |
| Boilerplate search page 2   | Ten pinned comparison cases: validation, audit selection, test setup, server configuration and plugin lifecycle                           | Selected implementation slices and licenses now assessed; exact browser-page membership remains unreproduced and no repository is claimed fully audited |
| react-boilerplate           | Paired source/test generation and variant linting                                                                                         | Future TinyTools multi-file patterns, modernized for current React/TypeScript                                                                           |
| electron-react-boilerplate  | Renderer-to-host smoke assertions and explicit subscription cleanup                                                                       | Existing Tauri tests and host-event adapters                                                                                                            |
| ChristianLempa/boilerplates | Typed/grouped parameters, bounded combination matrices and explicit secret metadata                                                       | Chunk inspection UI and future versioned schema extensions; do not bulk-import its legacy library                                                       |
| aspnetcorehero/Boilerplate  | Permission/audit display and application/domain/infrastructure boundaries                                                                 | UI design reference and selected future service templates; no C# runtime adoption                                                                       |

Each decision above is an engineering recommendation. The linked reviews distinguish directly read implementation from README claims, discovery-only entries and future acceptance work.

## Inspiration is a first-class result

This review asks three different questions of each source: what implementation could be reused with the right license and owner; what idea can be specified independently as native behavior; and what richer capability emerges when that behavior is combined with existing primitives. A Markdown skill can be a valuable source for criteria, workflow state, examples or user interaction even when it contains no reusable runtime. A small mechanism may become more useful in combination than it is as a standalone port.

The factual boundary stays clear. A source's prompt that asks an agent to verify a result is evidence of a proposed method, not evidence that its runtime enforces verification. The derived design can turn that method into typed requirements and observed artifact checks. That is a new engineering proposal and must carry its own tests. License restrictions constrain copying and distributing source, text, assets or dependencies; they do not automatically discard an independently specified concept. Any proposed reuse still needs the selected file's actual license and notices.

| Original source family      | Independent native capability inspired by it                                                | Broader composition worth assessing                                                                                              |
| --------------------------- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| K-Dense-AI                  | Resource assumptions, artifact identities and scientific uncertainty become typed records   | An optional research workspace combines planning, bounded data inspection and reviewable evidence                                |
| science-superpowers         | Research questions, preregistered decisions and verification criteria become explicit state | A research workflow can compare planned analysis, observed execution and deviations without inventing public timestamp authority |
| claude-scientific-writer    | A document-neutral deliverable graph replaces filename-based completion guesses             | Authoring, compilation, figure inspection and citation checking can share artifact provenance                                    |
| scientific-agents           | Selected Markdown roles inspire domain-specific checklists and evidence requirements        | On-demand skill content can configure a native workflow while existing tool grants remain unchanged                              |
| agentic-data-scientist      | Review generations and terminal states make stale approval and empty criteria visible       | A bounded plan/implement/review loop can produce an honest task outcome rather than another independent agent runtime            |
| multica-ai                  | Portable package identity, collision handling and explicit install outcomes                 | A native skill-bundle manager can combine digest acknowledgement, optional assets and reproducible provenance                    |
| pstack                      | A pure projection of repository/check state drives explainable waiting and next actions     | Durable coding handoffs can resume verification on the same head without treating missing checks as success                      |
| ContextLattice              | Selected and omitted evidence carry budget and freshness explanations                       | TinyMemory recall and TinyAgents context assembly can form a reproducible evidence packet                                        |
| Victory                     | Accessible selection state and explicit visible-domain data revisions                       | A research result viewer can link plots, tables, source rows and provenance without replacing ordinary dashboards                |
| trajectory                  | Codec diagnostics and normalized event identity preserve heterogeneous session history      | Session import, recall and handoff continuity can share loss-aware records                                                       |
| actix-net                   | Capacity ownership and drain outcomes become explicit lifecycle contracts                   | Connector servers, agent dispatch and resource-aware admission can share compatible accounting and shutdown evidence             |
| Boilerplate search sample   | Narrow mechanisms such as validation order, test lifecycle and contract-shaped outputs      | A catalog of independently specified patterns can supply reusable templates with precise failure cases                           |
| react-boilerplate           | Paired implementation/test generation becomes a versioned change plan                       | Multi-file chunk application can preview a coherent change set and track verification debt                                       |
| electron-react-boilerplate  | A host/renderer smoke contract and explicit subscription disposal                           | Native desktop validation can connect launch, bridge behavior, cancellation and cleanup                                          |
| ChristianLempa/boilerplates | Typed parameter metadata drives discoverable, validated forms                               | A chunk inspector can combine preview, constrained variant testing and safe secret references                                    |
| aspnetcorehero/Boilerplate  | Permission and audit concepts become understandable product surfaces                        | A shared action model can align UI availability, host admission and observed outcome                                             |

The linked reports refine these trajectories per selected repository and identify where the evidence is only packaging or guidance. The ten additional search-sample entries each have their own source/idea/failure assessment. No “more code means more value” ranking is intended: priority follows a concrete product gap, explanatory power, implementation cost and evidence needed.

## Victory: composable analytical interactions

Inspected commit: [`75549cf8c67e3ea220f862eb7f530c36d540d41c`](https://github.com/FormidableLabs/victory/commit/75549cf8c67e3ea220f862eb7f530c36d540d41c). The inspected package declares version 37.3.6, a React peer requirement of `>=16.6.0`, and separately packaged charts, containers and shared events. GitHub's license metadata is inconclusive, but the actual [LICENSE.txt](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/LICENSE.txt) is MIT and includes contributor notices. [Package manifest](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory/package.json)

### Existing overlap

OpenHuman already uses Recharts `^2.15.0`; the cost and token dashboards share a tooltip component, localized labels and explicit empty-domain handling. Their unit tests mock chart primitives because DOM-only tests lack layout measurement. This supports a focused interaction/accessibility improvement and a real-browser check, not a claim that the application lacks charts. [Current package](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/app/package.json), [token chart](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/app/src/components/dashboard/TokenUsageChart.tsx), [chart tests](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/app/src/components/dashboard/TokenUsageChart.test.tsx)

| Scale       | Candidate                                                                             | Adaptation and useful acceptance evidence                                                                                                       | Estimated effort |
| ----------- | ------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------- | ---------------- |
| Tiny detail | Stable chart title/description IDs and ARIA linkage                                   | Add an accessible summary and data-table alternative to existing dashboards; test multiple charts, empty data and keyboard/screen-reader names  | 0.5–1.5 days     |
| Primitive   | Controlled visible-domain state and bounded data selection                            | Keep filtering, zoom state and the displayed aggregate tied to one data revision; preserve units, timestamps and missing samples                | 2–4 days         |
| Widget      | Linked usage/cost brush with a nearest-point tooltip                                  | Compare current Recharts capabilities before adding a dependency; test pointer, touch, resize, keyboard alternatives and synchronization        | 3–5 days         |
| Feature     | Research result view with scatter/error bars, distributions and selected observations | Use a typed data/units/provenance contract and explicit uncertainty; test real rendering and export                                             | 1–2 weeks        |
| Module      | A scoped Victory-based analytical viewer                                              | Measure bundle, render and interaction cost on representative data, isolate the dependency to that view and preserve current dashboard behavior | 2–4 weeks        |

These are contributor estimates including focused tests, not measured delivery promises.

### What the source contributes

The [base container](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-core/src/victory-container/victory-container.tsx) connects optional titles/descriptions to generated IDs, ARIA attributes and an SVG role. That is a concrete accessibility pattern. A chart's labels alone do not make every visual interaction keyboard-accessible; test the actual product workflow.

The [zoom container](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-zoom-container/src/victory-zoom-container.tsx) can select the visible domain and downsample it; the default target when downsampling is explicitly enabled is 150 points. The [zoom helpers](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-zoom-container/src/zoom-helpers.ts) bound zoom around minimum domains and throttle interaction handlers. These are useful design references, but downsampling can conceal narrow peaks or uncommon observations. A scientific view must disclose decimation and retain access to the underlying records; a point count is not a semantic error bound.

The [container composer](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-create-container/src/create-container.tsx) combines zoom, selection, brush, cursor and Voronoi behavior through hooks and event mutations. Its event-merging helper explicitly does not handle `eventKey` or `childName`. That limitation matters for complex linked dashboards: adopt a narrowly tested composition rather than assuming arbitrary containers combine correctly.

No Victory runtime or performance benchmark was executed during this audit. The package metadata's React range also does not prove every interaction works under OpenHuman's React version. Preserve MIT notices for copied or substantially adapted implementation; review the selected package's dependency licenses if using it directly.

### Native design that preserves analytical meaning

A small accessibility increment should start with the existing Recharts wrappers. The useful contract is one stable chart identity with an accessible title, a concise textual summary and a table of the selected data. Victory stores a generated container ID across renders and composes title/description references with supplied ARIA references. Copying that code is optional; the behavior can be specified and implemented through the current React conventions. Server rendering, multiple instances and remounting need deliberate ID handling. [Container identity and ARIA composition](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-core/src/victory-container/victory-container.tsx)

A richer analytical view needs a dataset contract before a chart dependency. Separate source revision, original row identity, units, missing values, aggregation rule, visible domain and selection state. The same selection should drive the visual highlight, accessible table, tooltip and exported subset. When the source revision changes, explicitly recompute or invalidate a selection instead of carrying pixel coordinates onto different observations.

| Layer           | Small independent design                                          | Larger combination                                                                            |
| --------------- | ----------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| Description     | Stable accessible name and explanation of units/coverage          | Screen-reader and export summaries reference the same selected data                           |
| View state      | Controlled domain with valid minimum extent and a reset action    | Linked cost, token and run-duration views compare the same time interval                      |
| Data projection | Bounded visible rows with original IDs and declared decimation    | A research plot can open the underlying table or provenance for any selected point            |
| Interaction     | One tested brush/zoom/pointer contract with keyboard alternatives | A multi-panel analytical workspace shares selection without sharing accidental event handlers |
| Evidence        | Source revision and transformation recipe remain visible          | An exported figure and its dataset can be verified against the same artifact inventory        |

The zoom implementation adds boundary neighbors to support connected lines and skips downsampling stacked data. Its visible selection uses ordered array searches and slicing. Those details create acceptance obligations for unsorted inputs, domains outside the data, boundary points and stacks; the library's existence does not prove those cases meet a product-specific meaning. Test whether the first and last relevant observations survive selection and whether missing-data gaps remain gaps. [Visible-data path](https://github.com/FormidableLabs/victory/blob/75549cf8c67e3ea220f862eb7f530c36d540d41c/packages/victory-zoom-container/src/victory-zoom-container.tsx)

**Proposed acceptance cases:**

1. Render several charts together, then remount one. Every title/description reference resolves to its own chart; the accessible summary retains units and time scope.
2. Exercise empty, one-point, duplicate-x, unsorted and nonfinite data. Normalize or reject according to a documented contract rather than silently drawing misleading geometry.
3. Zoom beyond both ends of the source domain. Preserve intended endpoint behavior and provide an explicit reset that restores the original scope.
4. Place a narrow outlier and a missing-value gap between decimated points. The UI must disclose decimation and retain access to the full data; if extrema preservation is a requirement, test it directly.
5. Select observations, then replace the source revision. The table, tooltip and export must use one revision or clearly invalidate the selection.
6. Combine brush and zoom on two child series with overlapping identifiers. Verify the composer limitation on child/event addressing rather than assuming arbitrary event merges work.
7. Test pointer, touch, keyboard-only and reduced-motion use in a real renderer. A DOM test with mocked layout cannot certify native-webview interaction.
8. Measure representative data volumes and resize cadence. Record input size, retained points, frame/interaction latency and bundle impact before proposing a dependency change.

The smallest result may be a descriptive chart wrapper and two rendering regressions. A larger Victory viewer should be a separately loaded feature only if the current chart library cannot meet a demonstrated interaction need. Neither route requires an all-dashboard migration.

## actix-net: readiness, accounting and shutdown semantics

Inspected commit: [`acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941`](https://github.com/actix/actix-net/commit/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941). The workspace declares Rust 1.89 and `MIT OR Apache-2.0`; per-crate license files are present. The inspected [workspace manifest](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/Cargo.toml) and [MIT license](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/LICENSE-MIT) are the evidence, rather than treating repository metadata as a complete license inventory.

### Existing overlap

OpenHuman's RPC server already uses Axum and Tokio, accepts an explicit cancellation token, performs graceful shutdown, and runs exit cleanup even after a serving error. Its classification layer already distinguishes backpressure. TinyAgents has bounded dispatch and cancellation. A new Actix runtime would overlap with these owners; the useful port is a specific state machine or regression that demonstrates a current gap. [RPC serving path](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/crates/openhuman-rpc/src/server/serve.rs), [classification](https://github.com/AmirrezaFarnamTaheri/openhuman/blob/0a0ac335a3ed7210797eeaa2691d6f843ee018a3/crates/openhuman-rpc/src/server/classify.rs)

| Scale        | Candidate                                                                            | Owner and adaptation                                                                                     | Estimated effort                                                  |
| ------------ | ------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Primitive    | Admission capacity increments before a connection becomes visible; release only once | Existing connection/run budgets, with drop/cancel/race regressions                                       | 1–3 days                                                          |
| Primitive    | Explicit readiness separate from request execution                                   | Current Tower/Tokio admission interfaces; preserve overload errors and avoid an unnecessary trait bridge | 2–4 days if a gap is reproduced                                   |
| Feature      | Drain outcome distinguishes graceful completion from forced timeout                  | RPC/TinyBus lifecycle, with in-flight requests, new-admission refusal and cleanup assertions             | 2–4 days                                                          |
| Test utility | OS-assigned loopback port with retained listener and explicit server cleanup         | Existing native test harness; avoid free-port probe then rebind races                                    | 0.5–1.5 days                                                      |
| Whole crate  | `actix-codec` or `actix-service` in a genuinely Actix-native module                  | Evaluate direct interoperability, MSRV, dependencies and thread model first                              | 1–2 weeks for a narrow adapter; no adoption currently recommended |
| Whole plane  | Replace the RPC/transport runtime                                                    | No demonstrated product benefit in this audit; substantial ownership and behavior migration              | Not recommended                                                   |

The [service trait](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-service/src/lib.rs) uses shared-reference `poll_ready` and `call`, supports `no_std` with allocation and has separate service factories. Readiness is documented as best effort: a subsequent call can still fail, and a caller may invoke `call` without polling readiness. An adapter must preserve those semantics. It must also respect that Actix commonly permits local, non-`Send` futures; these are not automatically interchangeable with OpenHuman's cross-thread futures.

The [worker implementation](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/worker.rs) documents the connection counter's ownership and makes the count visible before publishing a connection. It separates unavailable, available and shutdown states, and reports whether shutdown completed gracefully or required a timeout. These are concrete invariants to compare with current budgets and shutdown receipts. Copying the server wholesale is unnecessary to use the test questions.

The [test server](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/test_server.rs) binds an OS-assigned loopback port and retains its server handle. The [TLS feature matrix](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-tls/Cargo.toml) supports several TLS generations; importing all of it would broaden the dependency graph. Select only the implementation actually needed, and retain the host's current certificate and credential policy.

### Translate lifecycle invariants, not the runtime

The counter is deliberately offset by one; the accept side increments before sending, a failed send reverses that increment, and the connection guard decrements on drop. Availability is also cached by the accept loop and refreshed through a wake notification when capacity returns. The protocol around the atomic value is part of the design. Copying only an increment/decrement helper would miss that ownership and wakeup relationship. [Counter contract and send path](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/worker.rs)

For current owners, represent capacity with a permit whose owner changes at dispatch. A task that is queued, admitted, handed off, cancelled or finished must have one clear accounting state. Readiness can suggest when dispatch is useful; it cannot authorize a request or promise that the next call succeeds. A future resource planner can inform a permit ceiling, but it cannot revoke a running task's authority by modifying an unrelated counter.

| Transition           | Required local invariant                                                 | Observable outcome                                            |
| -------------------- | ------------------------------------------------------------------------ | ------------------------------------------------------------- |
| Reserve capacity     | No work is externally visible before accounting accepts it               | Admission or overload result                                  |
| Transfer to worker   | Failed transfer returns/relinquishes the reservation exactly once        | Dispatch failure with no leaked capacity                      |
| Execute and cancel   | The active owner releases capacity when execution stops                  | Completion/cancellation state retains whether work had begun  |
| Begin drain          | New admission closes while existing permitted work is tracked            | Draining with an in-flight count                              |
| Deadline expires     | Remaining work is cancelled according to existing policy                 | Forced shutdown, retained incomplete count and cleanup result |
| Startup is abandoned | A worker that cannot report successful startup must not continue serving | Startup failure, no orphan service                            |

The inspected worker also exits if its success report cannot be delivered because startup was abandoned. That is a useful inspiration for module/connector boot lifecycles: a disconnected requester should not leave a service running merely because initialization succeeded locally. Implement it at the owner of the existing lifecycle, with that module's cancellation and lease rules. [Abandoned startup handling](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/worker.rs)

**Proposed acceptance cases:** saturate a capacity of one; fail the channel transfer after reservation; cancel before handoff; cancel after the worker owns the permit; drop a response consumer; abandon startup before its readiness report; and reach the drain deadline with one active connection. Each should have deterministic permit accounting, no false graceful result and no indefinite shutdown wait. Use the existing cancellation-aware test infrastructure and controlled scheduling, not sleeps that merely reduce race frequency.

The peer test API makes a valuable distinction between an unused address and an owned listener. Its address-only helper explicitly drops the listener and therefore releases the port; the listener-returning helper keeps the reservation alive. The server handle joins a dedicated thread on drop, which is not a universal cleanup pattern for asynchronous test code. Adopt retained-listener ownership while fitting shutdown to the current Tokio test harness. [Test-server ownership](https://github.com/actix/actix-net/blob/acb5b7d3cc65c7f9e92af7985ca4ae2084ae5941/actix-server/src/test_server.rs)

A practical first increment is a current-owner regression that reproduces a lost permit, premature drain report or port rebind race. If no gap appears, the review still contributes explicit lifecycle requirements for future modules. A complete Actix crate may fit a separate Actix-native service, but adding it to the main host without such a need would create adapter and executor complexity without demonstrated benefit.

## Suggested implementation sequence

1. Complete the assembled host gates for the published structural-code, permission and actual-execution contribution. TinyTools and TinyAgents owner results already exist; they do not substitute for the host composition result.
2. Prototype two transcript codecs in TinyMemory. Require stable full/chunked identity, duplicate-safe replay, tool/result linkage, loss accounting and namespace enforcement.
3. Add selected/omitted evidence receipts around existing recall. Explicit zero budget stays zero; final serialization must fit or refuse; approximate counts stay labeled approximate.
4. Extend provenance from trusted write receipts and observed process results. Keep inferred shell effects and caller-supplied claims distinguishable. Add a reviewer export only after source identity and omissions are specified.
5. Add a small chunk-parameter inspector and bounded generation matrix. Treat multi-file publication, secret references and dependency installation as separate contracts.
6. Offer optional research profiles and a bounded dataset inspector. Keep scientific interpretation and preregistration claims tied to actual evidence.
7. Add chart and network refinements only when product measurements or a reproduced defect justify them.

The detailed reviews include larger module alternatives so they remain assessable. Priority here reflects fit and demonstrable gaps, not repository popularity. A proposal becomes a port only after its owner, source/notice record, input/output contract, migration implications and behavioral acceptance evidence are concrete.

## Compositions that exceed any single borrowed mechanism

These are independent product designs derived from the reviewed mechanisms. They are deliberately more ambitious than literal source extraction, while retaining small first increments.

### A coding continuity and verification workspace

Combine trajectory-style loss-aware import, TinyMemory's scoped storage, ContextLattice-style evidence selection receipts, pstack-style exact-revision repository state, and the current bounded handoff. The feature would answer: what is known, where did it come from, which revision was tested, what was omitted, and what action remains?

A minimum demonstrator should import one real format, recall within one authorized workspace, and hand off a packet that distinguishes observed checks from claims. It should survive interruption and replay without duplicating records or promoting a stale test. Its acceptance scenario is a developer resuming work after both the repository and the conversation changed. The expected result is an explainable partial state, not a fabricated seamless memory.

### A research workbench with reviewable outputs

Combine a Markdown research method's questions and decision points with native resource planning, bounded dataset profiling, typed review state, observable tool results and a static reviewer export. Victory-inspired linked tables/plots can expose the selected observations and uncertainty. This makes an instruction such as “inspect the residuals before claiming success” actionable: the workflow can require a particular artifact and record whether the user or an admitted tool inspected the current revision.

The first version should support one documented workflow rather than import a persona catalog or scientific stack. Its acceptance scenario includes a failed analysis step, a revised dataset, an unapproved stage and a missing figure. The workbench must retain all four limitations in the exported record. It cannot infer scientific validity solely from a completed command or a model's approval.

### A reusable-code studio

Combine chunk identities, typed parameter forms, paired implementation/test templates, bounded variant generation and explicit multi-file publication semantics. Boilerplate projects contribute ideas about required output sets and developer experience; their old frameworks need not be copied. TinyTools should own pure rendering and bounded native file operations, while OpenHuman presents previews, reviews constraints and connects actual verification.

The minimum useful version is a parameter inspector for the current single-file contract. A larger feature needs a versioned multi-file plan and partial-failure semantics before it can safely claim transaction-like behavior. Its acceptance scenario changes a manifest or destination after preview, then cancels midway through staging. No stale or unauthorized final publication should occur, and any already-published subset must be explicit.

### Resource-aware, explainable execution

Combine the resource planner's binding limits with Actix-inspired permit ownership, the current TinyAgents target observation and existing host cancellation. The result is not a new scheduler by default: it is an execution explanation that connects an estimated workload, actual admission and observed termination.

Start by showing why work waited or was refused. Only then consider adaptive capacity with measured workload estimates. Its acceptance scenario has available agent slots but insufficient memory for a compiler, followed by cancellation of another task. The host should release the right capacity once and explain whether the waiting task was merely recommended, admitted, started or completed.

## Evidence and decision maintenance

Keep source observations, independent design proposals and implementation receipts separate when these reports evolve. A future contribution should record the owner issue/PR, exact source inspiration, any reused file and notices, the native contract, the focused tests and the product benefit being measured. A failed hypothesis remains useful evidence; it should not be relabeled “complete” because a different component has passed CI.

The original source pins above are immutable review snapshots. They should not silently move when a peer's default branch changes. The additional Meta-project dossiers broaden the research, while these four reports remain the detailed entry point for the originally requested families and their cross-project combinations.
