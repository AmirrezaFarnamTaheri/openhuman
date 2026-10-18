# PR 14: implementation review and roadmap disposition

This review covers PR [#14](https://github.com/AmirrezaFarnamTaheri/openhuman/pull/14), its review discussion, and the supplied **AST and Boilerplate Engine Roadmap** dated 2026-10-17. The [CodeRabbit reconciliation](pr14-coderabbit-review.md) maps all 18 unique bot findings to their fixes and validation evidence. This review distinguishes working code from architectural proposals and benchmark targets. The original PR included a large upstream merge; neither a filename count nor the number of workflow YAML files measures duplicated runtime work.

## Continuation on 2026-10-18

The integrated upstream baseline is
[`tinyhumansai/openhuman:main` at `3d74e06ee0233e48752b892fab24ae37ccad906c`](https://github.com/tinyhumansai/openhuman/commit/3d74e06ee0233e48752b892fab24ae37ccad906c).
It includes the TinyHumans backend extraction, storage schema v2, MCP cleanup
fences, credential fallbacks and upstream CI routing and storage checks. This
range has no `app/src` changes; the PR's onboarding and navigation changes remain
in the integrated tree. Conflict resolution preserves separate candidate-key
probes, hosted catalog registration and managed/local CortexDB credentials.

The generic structural-code and execution-observation changes are now published
in their owning repositories. The contributor's composition uses an explicit
fork URL and exact gitlinks; the canonical contributions remain separately
reviewable. There is one nested TinyTools checkout.

| Contribution                                                                                | Published revision                         | Current validation                                                                                                                                                                                                                                               |
| ------------------------------------------------------------------------------------------- | ------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [TinyTools #74](https://github.com/tinyhumansai/tinytools/pull/74)                          | `daec6c7a563336dd83d2e65ae38b2b169179aada` | [Final owner CI passed](https://github.com/tinyhumansai/tinytools/actions/runs/39129804642), including the unchanged 90% per-source-file coverage gate across 156 executable files. Focused local regressions passed 43 pure-engine and 35 native-adapter cases. |
| [TinyAgents #492](https://github.com/tinyhumansai/tinyagents/pull/492)                      | `e4021670f652ba0ec59c244aad4f660f216c455b` | [All ten current-head checks passed](https://github.com/tinyhumansai/tinyagents/actions/runs/39131507258), including the complete feature matrix, Windows and 89.14% line coverage. The generic contribution retains its original TinyTools pin.                 |
| [TinyAgents fork composition #1](https://github.com/AmirrezaFarnamTaheri/tinyagents/pull/1) | `69eb675a64ab0317531ce37a9a117c0e09dc42cc` | [All ten composed-chain checks passed](https://github.com/AmirrezaFarnamTaheri/tinyagents/actions/runs/39131512770), with 89.14% line coverage. This branch merges canonical TinyAgents main at `b81a455d` and pins the TinyTools revision above.                |

Both TinyAgents PRs are ready for review. Passing their matrices establishes
owner and composed-library evidence; it does not establish a passing OpenHuman
application build, host integration target, changed-line coverage result or
native platform smoke.

Released-module compatibility is tracked separately from source gitlinks:

- **TinyMCP:** the immutable [v0.19.1 release](https://github.com/tinyhumansai/tinymcp/releases/tag/v0.19.1)
  supplies wire contract **1.26**. All 17 archive records match its published
  checksum manifest. The required `4ef02dc9` source pin adds the host-side bus
  cleanup table and its tests after that tag; the narrow pin exemption records
  that difference.
- **TinyBox:** the immutable [v0.1.36 release](https://github.com/tinyhumansai/tinybox/releases/tag/v0.1.36)
  supplies wire contract **1.21**, matching the retained `8cb2a90c` source
  contract. All 17 registered archives were checked against their official
  checksums and asset digests. The earlier 0.1.35 artifact could not satisfy that
  source contract and caused module-admission failures in shell-based tests.
- **TinyFlows:** the supported [v0.8.23 release](https://github.com/tinyhumansai/tinyflows/releases/tag/v0.8.23)
  implements Schedule **3**. The pinned `cad43f97` source declares Schedule
  **4**, whose added optional `StartScheduler.config.now` field the host does
  not send. The compatibility qualification retains all admission and version
  checks. The new [released-module integration target](../../tests/tinyflows_release_compatibility.rs)
  verifies the actual released method set, manual execution, stored history,
  due-job dispatch and the agent-only gate; its current host-native run is
  **pending**, so source inspection alone is not recorded as runtime proof.

The current OpenHuman assembly includes the owner composition, host registry,
packed-call policy, finish evidence and structured handoff integration. Its
combined native targets, application CI, full pre-push and final coverage remain
pending at this document checkpoint. The older
[OpenHuman run at `11171dac`](https://github.com/AmirrezaFarnamTaheri/openhuman/actions/runs/39127887055)
has known Rust-lint, storage and checks-lane failures; its coverage lane was
still running when this record was refreshed. Earlier passes and failed
resource-limited attempts are historical receipts, not certification of this
new assembly.

## Concrete review findings

| Finding                                                                 | Change and expected behavior                                                                                                                                                                                                                                                                              |
| ----------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Fork CI could remain pending or select an inactive caller               | The hosted workflow preserves the stable `Lanes` caller recognized by the gate already running on the default branch. `scripts/ci/ci-gate.mjs` also recognizes historical caller names and propagates routing failures. Workflow tests derive names from YAML and evaluate the actual routing predicates. |
| Fork audit confused transport failures with request-budget exhaustion   | `scripts/ci/upstream_fork_audit.py` checkpoints failed work, continues independent forks, and resumes incomplete enumeration without reporting a false complete census.                                                                                                                                   |
| Preview regression checked any matching YAML line                       | The test parses `jobs['ui-preview'].if` using a direct, pinned `yaml` dependency.                                                                                                                                                                                                                         |
| Every push paid for unrelated application checks                        | `.husky/pre-push` scopes application and Rust gates to the pushed paths, retains conservative behavior for unresolved ranges, and keeps `PRE_PUSH_FORCE_FRONTEND=1` / `PRE_PUSH_FORCE_CLIPPY=1`. Hooks do not rewrite source files.                                                                       |
| Chip tabs invoked selection twice                                       | Radix activation notifications are deduplicated while preserving click-only activation. Keyboard and pointer regressions cover the behavior.                                                                                                                                                              |
| Tour and settings discovery depended on English                         | Commands and advanced-settings labels use actual locale values. New UX strings and inherited AI fleet/feasibility strings are translated across the shipped locales.                                                                                                                                      |
| A legacy walkthrough key could start a tour unexpectedly                | Legacy pending state no longer overrides the opt-in tour entry point.                                                                                                                                                                                                                                     |
| Discard-and-back could retain a navigation bypass                       | The missing JSX brace is repaired and bypass lifetime is tied to the actual history transition. Direct-entry routes retain a safe exit.                                                                                                                                                                   |
| Dismissing the last notice lost focus                                   | The notice tray provides a stable focus destination. Announcement dismissal and account changes have regression coverage.                                                                                                                                                                                 |
| Announcements interrupted startup and drafts                            | Announcements are a notice source, preserving severity, action links and per-user dismissal while avoiding a fullscreen promotional gate.                                                                                                                                                                 |
| The data router was initialized by mutating a global during render      | `main.tsx` now creates the router after hash and user bootstrap and supplies it to `App`. StrictMode does not create duplicate history listeners, and auxiliary windows do not create the main router.                                                                                                    |
| A localhost memory URL was mistaken for an app-managed service          | `managed_local` is explicit configuration. A manually configured local server keeps its own key; switching endpoints clears managed ownership.                                                                                                                                                            |
| Local memory startup could overwrite newer settings                     | Startup reloads the latest configuration after provisioning, anchored to the original configuration path and workspace. Concurrent model changes and account switches do not redirect the save.                                                                                                           |
| Editing managed memory could reuse the wrong credential                 | The manual-connection form requires a manual key when leaving managed mode. Cancelling preserves the managed connection; an existing manual connection can still reuse its own saved key.                                                                                                                 |
| Docker failures lost operation context or exposed raw details           | Readiness, image, credential and startup failures use distinct, sanitized messages. The UI retains actionable error information and blocks conflicting setup actions.                                                                                                                                     |
| Production child-page dynamic imports exceeded the documented exception | Skills and Memory use static child imports. Existing literal route-level splitting remains in `AppRoutes.tsx`.                                                                                                                                                                                            |
| A model-provider URL guard failed the full lint gate                    | An explicit ASCII character-code check preserves rejection of NUL, other control characters and DEL without a prohibited control-character regular expression. Regression tests cover the credential boundary.                                                                                            |
| Dependency diagram contradicted the dependency text                     | `AGENTS.md` and `CLAUDE.md` now show callers pointing to dependencies: hosts → RPC → TinyHumans → embed → core.                                                                                                                                                                                           |
| Existing Rust tests did not compile or satisfy layout gates             | String borrows, an omitted source-evidence import and an ambiguous conversion are corrected. Long and inline tests are separated by responsibility without changing production policy.                                                                                                                    |

## Structural code and reusable-code foundation

The generic implementation belongs to **TinyTools**, nested through `vendor/tinyagents/vendor/tinytools`, and is published in [TinyTools #74](https://github.com/tinyhumansai/tinytools/pull/74). OpenHuman composes its native adapter through the separately validated TinyAgents fork chain above. Host policy, registry and integration tests remain OpenHuman responsibilities; the pending application gates must pass before a release claim.

### Library and execution boundaries

- `tinytools-code` is a source-in/source-out library. It contains grammars, queries, parsing, structural edits, template contracts and rendering. It does not read project files, install dependencies, execute generated code or decide filesystem permissions.
- The optional `tinytools-std/code` feature supplies six native tools through the existing `FsGate` and trusted run-workspace context. The tools share a parser cache and are deferred from the initial tool schema surface.
- The OpenHuman adapter composes the tools into its existing registry and coding pack, preserving one TinyTools checkout and one identity for the shared `Tool` types. Its current integration-test result is pending.

| Tool            | Contract                                                                                                                                                                                                         |
| --------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `ast_outline`   | Declarations, folded source, lexical calls/imports, syntax diagnostics, revision and explicit continuation offsets. Level 0 omits folded source; level 1 includes it.                                            |
| `ast_symbol`    | One uniquely selected declaration, identified by name, qualified name or a revision-bound ID. Large source results are paged at UTF-8 boundaries.                                                                |
| `ast_patch`     | Revision-checked body/symbol replacement, import insertion, member insertion or wrapping. Invalid original or resulting syntax is rejected.                                                                      |
| `chunk_search`  | Bounded, deterministic metadata search. Default five results; template source is not opened.                                                                                                                     |
| `chunk_inspect` | Parameters, slot contracts, target kind, dependencies and restrictions, with a metadata revision. No template source.                                                                                            |
| `chunk_apply`   | Materializes a file, replaces a selected function body, or appends a member. It returns a bounded selected source page with the revision and publication receipt. Existing targets require the current revision. |

The compiled grammar set is Rust, TypeScript, TSX, JavaScript/JSX, Python, Go, JSON and TOML. Query packs have a common capture vocabulary. Trees use bounded process-local caching; an edit computes byte and point changes, edits the previous tree, then reparses. Each analysis reports a cold parse, cache hit or incremental update; retained entry and source-byte counts are observable. Tests check eviction behavior.

The starter catalog contains nine complete-file patterns and three composable snippets. Examples include a fallible Rust function, a Tauri command shell, a typed TypeScript function, an accessible TSX action, Python cleanup, a Go HTTP handler, a Vitest suite, JSON/TOML configuration, checked arithmetic, a nullable guard and a struct field. Their dependencies are metadata for review; syntax validity alone does not prove that their imports or types are available in a project.

Custom patterns require an explicit `manifest_path` in `chunk_inspect` and `chunk_apply`; `chunk_search` searches the twelve bundled patterns. There is no automatic `.code-chunks` discovery. The manifest and its relative template path both pass `FsGate`, and the resolved template must remain under the manifest directory. Content-bound `chunk_revision` detects changes to either file. Manifest validation rejects duplicate IDs and unsafe paths. Parameters have explicit kinds; string values are escaped, identifiers are checked, and type expressions are parsed in their actual syntactic context. Standalone comment slots are expanded once. Inserted text is never rescanned for template directives. The final output is validated again, including configured forbidden AST nodes.

Applied source is ordinary source code. Agents can edit it immediately, including through `ast_patch`; there is no persistent template lock or hidden regeneration that overwrites later creative work.

### Editing guarantees and limits

Editing an existing file requires its current content hash. Every mutation requires valid syntax, host authorization, an allowed path and an available action budget. Native operations use canonical workspace paths, reject leaf symlinks, serialize concurrent code-tool edits and publish through a same-directory temporary file. New-file publication cannot overwrite an existing file. Existing file permissions are retained. Receipts are size-checked before mutation.

The host's shared file-state lock is also honored when installed. These locks coordinate cooperating tool calls. They do not make a portable compare-and-swap transaction against arbitrary external editors or malicious concurrent filesystem replacement. Re-read before retries and inspect the resulting revision.

The owner regressions cover bounded source pages, stale target and custom-template revisions, invalid syntax, leaf-symlink rejection, no-clobber creation, read-only previews, autonomy changes and action-budget exhaustion. Temporary-file preparation may run on a blocking worker, but final publication does not remain queued there: after the last asynchronous path/revision check, the adapter rechecks write authority and performs a short synchronous atomic publication while holding the cooperating write locks. Cancellation before that point drops the staged file. A successful publication records the write. The complete receipt is size-checked before action reservation or staging, and error truncation preserves UTF-8 boundaries. These claims follow the [native publication implementation](https://github.com/tinyhumansai/tinytools/blob/daec6c7a563336dd83d2e65ae38b2b169179aada/crates/tinytools-std/src/code/publish.rs) and [documented adapter contract](https://github.com/tinyhumansai/tinytools/blob/daec6c7a563336dd83d2e65ae38b2b169179aada/crates/tinytools-std/src/code/README.md).

Tree-sitter checks grammar, not types, intent, authorization logic, dependency availability or behavior. Call references are lexical observations, not a resolved semantic call graph. Structural edits preserve untouched byte ranges; they do not run a formatter or compiler. Use the repository's normal formatter, focused tests and type checker afterward. Unsupported declarations or languages fail explicitly.

## Disposition of the broader roadmap

The source audit does not support applying the supplied example patches verbatim. The boot example invents an endpoint and keyring API, hardcodes persistence readiness, and replaces the route outlet. The proposed task-card replacement also discards existing behavior and keyboard semantics. The following records each proposal family and its actual disposition.

| Proposal family                                                        | Disposition and evidence                                                                                                                                                                                                                                           |
| ---------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Incremental AST, query packs, outlines, targeted reads and patches     | Implemented in the owning TinyTools contribution with bounded inputs, caches, diagnostics and output. Native adapter and host registration are separate changes.                                                                                                   |
| Serialized on-disk syntax trees                                        | Not represented as supported. Trees are process-local parser objects; a future persistent derived index needs parser/query versioning, eviction and host storage policy.                                                                                           |
| Arbitrary dynamic WASM/shared-library grammars                         | Replaced by a compiled, explicit language allowlist for this foundation. New grammars require versioned query fixtures and parser tests. No executable grammar is downloaded from a project catalog.                                                               |
| Semantic scopes, transitive local types and resolved cross-file graphs | Lexical extraction is available; semantic resolution remains with LSP/compiler tooling. `source_graph.rs` remains explicit about its conservative evidence model.                                                                                                  |
| Canonically sorting imports                                            | Existing order is preserved. Import evaluation order can carry behavior in JavaScript and Python. Insertion is idempotent for the same parsed declaration; it is not an import optimizer.                                                                          |
| Progressive discovery, contracts, direct writes and microchunks        | Implemented. Raw templates stay out of system prompts and metadata results; selected code is loaded only for application.                                                                                                                                          |
| Claimed percentage token, latency and first-pass correctness gains     | Benchmark targets, not achieved results. A reproducible parser/view benchmark reports timings and bytes; production agent-token measurements require a fixed tokenizer and task corpus.                                                                            |
| Duplicate `.aui.tsx` components                                        | The inspected pairs already use a presentation component plus a thin Assistant-UI adapter. Deleting adapters would remove integration rather than remove a second renderer.                                                                                        |
| Replacing the composer/IME bridge with a new editor architecture       | The current bridge is a short synchronization boundary with dedicated IME tests. No reproduced defect justifies replacing editor transactions in this pass.                                                                                                        |
| Worker Markdown and variable-height thread virtualization              | A separate performance change requiring measured long-thread traces, search/selection/focus behavior and streaming fixtures. Presence of many thread tests is not evidence that all of their code rerenders per token.                                             |
| Broad inbound/outbound prompt-injection firewall                       | Preserve backend tool authorization and untrusted-data handling. A frontend regex does not establish a security boundary; expanding policy belongs to its owning runtime and contracts.                                                                            |
| Collapse every startup gate                                            | Announcements no longer block. Real persistence, credentials, account and daemon readiness remain authoritative. `ServiceBlockingGate` already retains mounted children; no fabricated readiness or automatic send is introduced.                                  |
| Merge all notification and approval semantics                          | Notices now include announcements and focus restoration. Approval, security and activity records retain their actual scopes; the work does not silently grant thread/flow-wide approval.                                                                           |
| Flatten settings, memory and connections                               | The PR's grouped navigation, settings search, connection hub and advanced controls are retained and repaired. Domain-specific forms keep their supported settings and deep links.                                                                                  |
| Automatic memory-provider migration                                    | Existing TinyMemory migration/import modules have checkpoint and retry contracts. Switching a provider does not imply consent to transfer all memories to a different service. Managed local startup now preserves explicit credentials and current configuration. |
| Complete memory capability trait and local fallback                    | Provider contracts belong to `tinymemory-api`; migration/import behavior belongs to TinyMemory's tools/integrations. No second host-side provider abstraction or silent remote-to-local migration is added.                                                        |
| Memory provenance links                                                | Existing source/lifecycle metadata is retained. A broader chat-to-memory inspector change needs a concrete source-record contract and UI acceptance tests.                                                                                                         |
| Schema-generated workflow forms and edge type checking                 | The UI already consumes `openhuman.flows_validate`, guards out-of-order results and applies local loop-cycle safety. Richer type compatibility belongs to TinyFlows' node/port contracts before UI generation.                                                     |
| Remove workflow navigation protection through autosave                 | The concrete stale-bypass and direct-entry issues are fixed. A local draft is not equivalent to a published graph; automatic persistence requires version/account scoping, quota failure handling and conflict recovery.                                           |
| Merge read-only/editable canvases and all run containers               | These hosts have different interaction contracts. A shared-renderer refactor needs explicit behavior parity rather than a target file-count reduction.                                                                                                             |
| Delta copilot editing and replay from a node                           | Requires owner-side versioned operations and checkpoint semantics. Arbitrary replay can repeat external side effects; cached upstream outputs alone are insufficient authorization or idempotency.                                                                 |
| MCP cryptographic tool pinning and process supervision                 | TinyMCP already persists fingerprint-matched tool catalogs and owns server definitions, secrets and connections. Definition-cache fingerprints are not claimed to be full semantic schema-change approval. Further hardening belongs upstream.                     |
| Unified permission engine and proactive OAuth refresh                  | Existing security, OAuth and connector owners remain authoritative. Notice recovery is improved without changing credential lifetime or broadening stored grants.                                                                                                  |
| Shared desktop/iOS route manifest                                      | Preserve platform-specific capabilities and routes. Desktop and iOS parity requires native WebView and pairing smoke tests before combining guards.                                                                                                                |
| SharedArrayBuffer, VAD and voice IPC changes                           | Requires cross-origin isolation, native WebView support and measured latency/bandwidth behavior. TinyVoice owns the native pipeline. No unsupported browser feature is assumed.                                                                                    |
| Six total CI workflows and one E2E script                              | Existing reusable lanes, path filtering, runner pools and artifact reuse do useful separation. Real fork-routing and retry defects are repaired; workflow-file count is not a throughput metric. Release trust boundaries are retained.                            |
| CI cache/linker improvements                                           | The image already installs sccache and mold; lanes configure cache reuse. Hook area-scoping now avoids unrelated checks without weakening relevant gates.                                                                                                          |
| Structured coding handoffs                                             | The prepared host integration supplies a bounded, versioned evidence contract for revisions, selected symbols, changed files and exact validation commands. It does not add unverified agent identities from the roadmap.                                          |
| SVG deduplication and blanket raster/audio conversion                  | The measured SVGs are not byte-identical copies. Different artwork is retained. Small audio files keep known playback behavior pending platform codec tests.                                                                                                       |
| Externalize embedded mascot                                            | Implemented by preserving the exact PNG bytes and importing an emitted asset URL.                                                                                                                                                                                  |
| Font subset loading and sourcemaps                                     | Unicode ranges already exist in `app/src/styles/fonts.css`. The existing sourcemap build policy remains the source of truth; release artifacts need inspection before claiming map removal.                                                                        |
| Idle animation and route splitting                                     | Decorative mascot/tetrahedron scheduling honors hidden/reduced-motion state; MeshGradient already had those guards. Route splitting stays within the documented root-router boundary.                                                                              |
| `codegen-units = 1`, new panic behavior and binary reduction targets   | The established release profile already has thin LTO, symbol stripping and 16 codegen units. A known project constraint rules out changing to one unit without evidence. No binary-size percentage is claimed.                                                     |

## Validation scope and remaining work

### Current owner evidence

The exact published owner revisions and CI runs are listed above. TinyTools'
43 pure-engine and 35 native-adapter regressions cover the new grammar,
rendering, paging and publication behavior. Its final owner CI also passed the
required workspace, feature, documentation, dependency, MSRV and per-file
coverage gates. TinyAgents' generic and composed revisions each passed all ten
checks. Their 89.14% line coverage exceeds the unchanged 80% owner gate.

The generic TinyAgents implementation was also exercised by the complete local
default harness suite: **2,845 tests passed**. The shared-adapter regressions
were first reproduced failing, and the five session-lock tests passed at
default parallelism after each test received its own process-wide destination.
That fixture correction preserves the production lock and the same-session
exclusion/cross-session independence assertions. The final feature matrices,
including PNG optimization and downstream JSON-order preservation, are the CI
evidence for the published revisions.

### Current OpenHuman verification boundary

The current combined native invocation includes
[`coding_handoff`](../../tests/coding_handoff.rs),
[`structural_code`](../../tests/structural_code.rs) and
[`tinyflows_release_compatibility`](../../tests/tinyflows_release_compatibility.rs).
Its result is pending. These targets are intended to exercise real host
registration and policy, direct and packed previews/writes, stale and denied
mutations, finish evidence, structured handoff round trips and the released
TinyFlows fixture. They are not replaced by tests of copied host implementations.

The inherited CI failures have prompted focused repairs to release metadata,
hosted-layer test imports/configuration, canonical storage-baseline selection
and explicit upstream boundary records. Temporary module-boundary and
process-static ownership records document remaining architecture work; they
are not claims that those boundaries have been fully extracted. The final
OpenHuman CI and whole-PR changed-line coverage must evaluate the assembled
source, including those repairs. No coverage threshold, module-admission
guard or native permission boundary has been relaxed to obtain a pass.

Application pre-push, the complete current test selection, combined coverage
and native platform smoke remain open at this checkpoint. A green dependency
matrix does not fill those entries. The final PR validation update must name
the published OpenHuman commit and its actual results.

### Historical evidence retained for review traceability

The [CodeRabbit reconciliation](pr14-coderabbit-review.md) preserves the 18
finding-to-fix mappings from the earlier PR revisions. Its 639-test frontend
selection, earlier 838-case script run, nine-case browser navigation run and
past root/desktop Clippy and pre-push receipts describe those revisions only.
The upstream-merge checkpoint
[`0a0ac335`](https://github.com/AmirrezaFarnamTaheri/openhuman/commit/0a0ac335a3ed7210797eeaa2691d6f843ee018a3)
also passed its actual pre-push before publication. None of those historical
counts is presented as a rerun of the current structural-code assembly.

Earlier unpublished owner commit IDs, bundle-size tables and parser timings
have been removed from the current evidence ledger. They are not reproducible
receipts for the newly published dependency chain. The maintained
[`structural` benchmark](https://github.com/tinyhumansai/tinytools/blob/daec6c7a563336dd83d2e65ae38b2b169179aada/crates/tinytools-code/benches/structural.rs)
reports fixed-fixture byte counts and cold/cache/incremental timings when run.
Token savings, startup latency, first-pass correctness and native power use
remain measurement targets. Establish a fixed task corpus, tokenizer, build
profile and platform before comparing them.

For an earlier PR preview that provisioned local memory before the explicit
ownership flag, **Start local memory** reuses the owned container, recovers its
credential and persists `managed_local` while keeping the named volume. Ownership
must not be inferred from a localhost URL: a manually operated server can use
that address.

## Integration and release sequence

1. **Published and validated:** TinyTools' pure engine and filesystem adapter
   in its own repository, with canonical PR #74 and the exact tested fork
   revision available for recursive checkout.
2. **Published and validated:** the generic TinyAgents contribution in
   canonical PR #492; a separate fork composition merges canonical main and
   records the explicit TinyTools source URL and exact gitlink.
3. **Current host work:** compose that chain in OpenHuman, keep generated code
   behind existing filesystem/security policy, and run the real host and
   released-module integration targets.
4. **Pending application gates:** complete current CI, relevant pre-push,
   changed-line coverage and supported native-platform smoke. Publish the
   resulting OpenHuman source and update its exact-revision receipts.
5. **Canonical integration:** maintain separately reviewable owner PRs and
   replace fork pins with canonical revisions once accepted. A contributor
   fork pin is reproducible source selection, not a claim of a canonical
   release or maintainer approval.

The roadmap dispositions above remain explicit. Deferred semantic indexing,
persistent syntax storage, workflow replay, automatic memory transfer and
native-platform changes need their own owner contracts and acceptance evidence.
They are not marked complete by the structural-code foundation.
