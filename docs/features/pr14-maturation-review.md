# PR 14: implementation review and roadmap disposition

This review covers PR [#14](https://github.com/AmirrezaFarnamTaheri/openhuman/pull/14), its review discussion, and the supplied **AST and Boilerplate Engine Roadmap** dated 2026-10-17. The [CodeRabbit reconciliation](pr14-coderabbit-review.md) maps all 18 unique bot findings to their fixes and validation evidence. This review distinguishes working code from architectural proposals and benchmark targets. The original PR included a large upstream merge; neither a filename count nor the number of workflow YAML files measures duplicated runtime work.

## Continuation on 2026-10-18

The current upstream integration incorporates `tinyhumansai/openhuman:main` at
`3d74e06ee0233e48752b892fab24ae37ccad906c`, including the seven mainline merges
since the previous shared ancestor. This brings in the TinyHumans backend
extraction, storage schema v2, MCP cleanup fences, credential fallbacks and the
current CI routing and storage checks. The upstream range has no `app/src`
changes; the PR's onboarding and navigation work remains in the integrated tree.

Conflict resolution keeps candidate API-key probes separate from the new hosted
catalog registry, so testing an unsaved credential cannot silently use a saved
key or session. The managed/local CortexDB credential split is retained. The
private CI lanes install their module fixtures after selecting the real test
command, including nextest, and a missing fixture fails before tests run.

The TinyMCP runtime registry is reconciled to the immutable
[v0.19.1 release](https://github.com/tinyhumansai/tinymcp/releases/tag/v0.19.1)
and all 17 entries in its published `checksum.toml`. That release carries wire
contract 1.26. The required upstream source pin `4ef02dc9` remains four commits
past the tag: those commits only change the host-side `tinymcp-bus` client
cleanup table and its tests. The exact pin exemption records that reason; it
must be removed when a release includes the cleanup fix.

The earlier unpublished TinyTools, TinyAgents and host-adapter commit IDs below
are historical records. Their source objects are unavailable in this workspace
and in the canonical remotes, so their old test counts do not verify the current
continuation. The owner changes are being rebuilt in isolated worktrees. They
must pass their owner gates and be published before OpenHuman consumes them;
the current host still uses published dependency pointers.

The focused merge script regressions pass 149/149. The first full native
`cargo check --locked -p openhuman -p openhuman-cli -p openhuman-tinyhumans --tests`
attempt reached the core test crate, then was killed at the workspace's 8 GiB
memory limit. Its stale unused import is corrected. That attempt is neither an
executed-test pass nor a successful test-compilation result. Current full script,
production Clippy and pre-push outcomes are recorded on the PR when this
increment is published.

## Concrete review findings

| Finding | Change and expected behavior |
| --- | --- |
| Fork CI could remain pending or select an inactive caller | The hosted workflow preserves the stable `Lanes` caller recognized by the gate already running on the default branch. `scripts/ci/ci-gate.mjs` also recognizes historical caller names and propagates routing failures. Workflow tests derive names from YAML and evaluate the actual routing predicates. |
| Fork audit confused transport failures with request-budget exhaustion | `scripts/ci/upstream_fork_audit.py` checkpoints failed work, continues independent forks, and resumes incomplete enumeration without reporting a false complete census. |
| Preview regression checked any matching YAML line | The test parses `jobs['ui-preview'].if` using a direct, pinned `yaml` dependency. |
| Every push paid for unrelated application checks | `.husky/pre-push` scopes application and Rust gates to the pushed paths, retains conservative behavior for unresolved ranges, and keeps `PRE_PUSH_FORCE_FRONTEND=1` / `PRE_PUSH_FORCE_CLIPPY=1`. Hooks do not rewrite source files. |
| Chip tabs invoked selection twice | Radix activation notifications are deduplicated while preserving click-only activation. Keyboard and pointer regressions cover the behavior. |
| Tour and settings discovery depended on English | Commands and advanced-settings labels use actual locale values. New UX strings and inherited AI fleet/feasibility strings are translated across the shipped locales. |
| A legacy walkthrough key could start a tour unexpectedly | Legacy pending state no longer overrides the opt-in tour entry point. |
| Discard-and-back could retain a navigation bypass | The missing JSX brace is repaired and bypass lifetime is tied to the actual history transition. Direct-entry routes retain a safe exit. |
| Dismissing the last notice lost focus | The notice tray provides a stable focus destination. Announcement dismissal and account changes have regression coverage. |
| Announcements interrupted startup and drafts | Announcements are a notice source, preserving severity, action links and per-user dismissal while avoiding a fullscreen promotional gate. |
| The data router was initialized by mutating a global during render | `main.tsx` now creates the router after hash and user bootstrap and supplies it to `App`. StrictMode does not create duplicate history listeners, and auxiliary windows do not create the main router. |
| A localhost memory URL was mistaken for an app-managed service | `managed_local` is explicit configuration. A manually configured local server keeps its own key; switching endpoints clears managed ownership. |
| Local memory startup could overwrite newer settings | Startup reloads the latest configuration after provisioning, anchored to the original configuration path and workspace. Concurrent model changes and account switches do not redirect the save. |
| Editing managed memory could reuse the wrong credential | The manual-connection form requires a manual key when leaving managed mode. Cancelling preserves the managed connection; an existing manual connection can still reuse its own saved key. |
| Docker failures lost operation context or exposed raw details | Readiness, image, credential and startup failures use distinct, sanitized messages. The UI retains actionable error information and blocks conflicting setup actions. |
| Production child-page dynamic imports exceeded the documented exception | Skills and Memory use static child imports. Existing literal route-level splitting remains in `AppRoutes.tsx`. |
| A model-provider URL guard failed the full lint gate | An explicit ASCII character-code check preserves rejection of NUL, other control characters and DEL without a prohibited control-character regular expression. Regression tests cover the credential boundary. |
| Dependency diagram contradicted the dependency text | `AGENTS.md` and `CLAUDE.md` now show callers pointing to dependencies: hosts → RPC → TinyHumans → embed → core. |
| Existing Rust tests did not compile or satisfy layout gates | String borrows, an omitted source-evidence import and an ambiguous conversion are corrected. Long and inline tests are separated by responsibility without changing production policy. |

## Structural code and reusable-code foundation

The generic implementation belongs to **TinyTools**, nested through `vendor/tinyagents/vendor/tinytools`. It is prepared in that owning repository, not copied into OpenHuman's core. The host integration is a separate adapter change. Publishing the owning contribution and updating the dependency chain are required before advertising this capability in a released OpenHuman build.

### Library and execution boundaries

- `tinytools-code` is a source-in/source-out library. It contains grammars, queries, parsing, structural edits, template contracts and rendering. It does not read project files, install dependencies, execute generated code or decide filesystem permissions.
- The optional `tinytools-std/code` feature supplies six native tools through the existing `FsGate` and trusted run-workspace context. The tools share a parser cache and are deferred from the initial tool schema surface.
- The prepared OpenHuman adapter composes the tools into its existing registry and coding pack. It must keep one TinyTools checkout and one identity for the shared `Tool` types.

| Tool | Contract |
| --- | --- |
| `ast_outline` | Declarations, folded source, lexical calls/imports, syntax diagnostics, revision and explicit continuation offsets. Level 0 omits folded source; level 1 includes it. |
| `ast_symbol` | One uniquely selected declaration, identified by name, qualified name or a revision-bound ID. Large source results are paged at UTF-8 boundaries. |
| `ast_patch` | Revision-checked body/symbol replacement, import insertion, member insertion or wrapping. Invalid original or resulting syntax is rejected. |
| `chunk_search` | Bounded, deterministic metadata search. Default five results; template source is not opened. |
| `chunk_inspect` | Parameters, slot contracts, target kind, dependencies and restrictions, with a metadata revision. No template source. |
| `chunk_apply` | Materializes a file, replaces a selected function body, or appends a member. It returns a receipt instead of generated source. Existing targets require the current revision. |

The compiled grammar set is Rust, TypeScript, TSX, JavaScript/JSX, Python, Go, JSON and TOML. Query packs have a common capture vocabulary. Trees use bounded process-local caching; an edit computes byte and point changes, edits the previous tree, then reparses. Each analysis reports a cold parse, cache hit or incremental update; retained entry and source-byte counts are observable. Tests check eviction behavior.

The starter catalog contains nine complete-file patterns and three composable snippets. Examples include a fallible Rust function, a Tauri command shell, a typed TypeScript function, an accessible TSX action, Python cleanup, a Go HTTP handler, a Vitest suite, JSON/TOML configuration, checked arithmetic, a nullable guard and a struct field. Their dependencies are metadata for review; syntax validity alone does not prove that their imports or types are available in a project.

Custom catalogs use bounded `*.chunk.json` manifests and relative template paths under `.code-chunks` or an explicitly selected allowed directory. Manifest validation rejects duplicate IDs and unsafe paths. Parameters have explicit kinds; string values are escaped, identifiers are checked, and type expressions are parsed in their actual syntactic context. Standalone comment slots are expanded once. Inserted text is never rescanned for template directives. The final output is validated again, including configured forbidden AST nodes.

Applied source is ordinary source code. Agents can edit it immediately, including through `ast_patch`; there is no persistent template lock or hidden regeneration that overwrites later creative work.

### Editing guarantees and limits

Editing an existing file requires its current content hash. Every mutation requires valid syntax, host authorization, an allowed path and an available action budget. Native operations use canonical workspace paths, reject leaf symlinks, serialize concurrent code-tool edits and publish through a same-directory temporary file. New-file publication cannot overwrite an existing file. Existing file permissions are retained. Receipts are size-checked before mutation.

The host's shared file-state lock is also honored when installed. These locks coordinate cooperating tool calls. They do not make a portable compare-and-swap transaction against arbitrary external editors or malicious concurrent filesystem replacement. Re-read before retries and inspect the resulting revision.

An independent adapter review added regressions for five boundary cases. Large-symbol paging now resolves metadata without first forcing the complete symbol through the exact-read limit. Preview and publication reject the same oversized output. Python body replacement preserves tab indentation and handles inline suites. A queued blocking publication retains both write locks and records the completed write even if its asynchronous caller is cancelled. Error responses are truncated at UTF-8 boundaries so a large invalid argument cannot bypass the response limit.

Tree-sitter checks grammar, not types, intent, authorization logic, dependency availability or behavior. Call references are lexical observations, not a resolved semantic call graph. Structural edits preserve untouched byte ranges; they do not run a formatter or compiler. Use the repository's normal formatter, focused tests and type checker afterward. Unsupported declarations or languages fail explicitly.

## Disposition of the broader roadmap

The source audit does not support applying the supplied example patches verbatim. The boot example invents an endpoint and keyring API, hardcodes persistence readiness, and replaces the route outlet. The proposed task-card replacement also discards existing behavior and keyboard semantics. The following records each proposal family and its actual disposition.

| Proposal family | Disposition and evidence |
| --- | --- |
| Incremental AST, query packs, outlines, targeted reads and patches | Implemented in the owning TinyTools contribution with bounded inputs, caches, diagnostics and output. Native adapter and host registration are separate changes. |
| Serialized on-disk syntax trees | Not represented as supported. Trees are process-local parser objects; a future persistent derived index needs parser/query versioning, eviction and host storage policy. |
| Arbitrary dynamic WASM/shared-library grammars | Replaced by a compiled, explicit language allowlist for this foundation. New grammars require versioned query fixtures and parser tests. No executable grammar is downloaded from a project catalog. |
| Semantic scopes, transitive local types and resolved cross-file graphs | Lexical extraction is available; semantic resolution remains with LSP/compiler tooling. `source_graph.rs` remains explicit about its conservative evidence model. |
| Canonically sorting imports | Existing order is preserved. Import evaluation order can carry behavior in JavaScript and Python. Insertion is idempotent for the same parsed declaration; it is not an import optimizer. |
| Progressive discovery, contracts, direct writes and microchunks | Implemented. Raw templates stay out of system prompts and metadata results; selected code is loaded only for application. |
| Claimed percentage token, latency and first-pass correctness gains | Benchmark targets, not achieved results. A reproducible parser/view benchmark reports timings and bytes; production agent-token measurements require a fixed tokenizer and task corpus. |
| Duplicate `.aui.tsx` components | The inspected pairs already use a presentation component plus a thin Assistant-UI adapter. Deleting adapters would remove integration rather than remove a second renderer. |
| Replacing the composer/IME bridge with a new editor architecture | The current bridge is a short synchronization boundary with dedicated IME tests. No reproduced defect justifies replacing editor transactions in this pass. |
| Worker Markdown and variable-height thread virtualization | A separate performance change requiring measured long-thread traces, search/selection/focus behavior and streaming fixtures. Presence of many thread tests is not evidence that all of their code rerenders per token. |
| Broad inbound/outbound prompt-injection firewall | Preserve backend tool authorization and untrusted-data handling. A frontend regex does not establish a security boundary; expanding policy belongs to its owning runtime and contracts. |
| Collapse every startup gate | Announcements no longer block. Real persistence, credentials, account and daemon readiness remain authoritative. `ServiceBlockingGate` already retains mounted children; no fabricated readiness or automatic send is introduced. |
| Merge all notification and approval semantics | Notices now include announcements and focus restoration. Approval, security and activity records retain their actual scopes; the work does not silently grant thread/flow-wide approval. |
| Flatten settings, memory and connections | The PR's grouped navigation, settings search, connection hub and advanced controls are retained and repaired. Domain-specific forms keep their supported settings and deep links. |
| Automatic memory-provider migration | Existing TinyMemory migration/import modules have checkpoint and retry contracts. Switching a provider does not imply consent to transfer all memories to a different service. Managed local startup now preserves explicit credentials and current configuration. |
| Complete memory capability trait and local fallback | Provider contracts belong to `tinymemory-api`; migration/import behavior belongs to TinyMemory's tools/integrations. No second host-side provider abstraction or silent remote-to-local migration is added. |
| Memory provenance links | Existing source/lifecycle metadata is retained. A broader chat-to-memory inspector change needs a concrete source-record contract and UI acceptance tests. |
| Schema-generated workflow forms and edge type checking | The UI already consumes `openhuman.flows_validate`, guards out-of-order results and applies local loop-cycle safety. Richer type compatibility belongs to TinyFlows' node/port contracts before UI generation. |
| Remove workflow navigation protection through autosave | The concrete stale-bypass and direct-entry issues are fixed. A local draft is not equivalent to a published graph; automatic persistence requires version/account scoping, quota failure handling and conflict recovery. |
| Merge read-only/editable canvases and all run containers | These hosts have different interaction contracts. A shared-renderer refactor needs explicit behavior parity rather than a target file-count reduction. |
| Delta copilot editing and replay from a node | Requires owner-side versioned operations and checkpoint semantics. Arbitrary replay can repeat external side effects; cached upstream outputs alone are insufficient authorization or idempotency. |
| MCP cryptographic tool pinning and process supervision | TinyMCP already persists fingerprint-matched tool catalogs and owns server definitions, secrets and connections. Definition-cache fingerprints are not claimed to be full semantic schema-change approval. Further hardening belongs upstream. |
| Unified permission engine and proactive OAuth refresh | Existing security, OAuth and connector owners remain authoritative. Notice recovery is improved without changing credential lifetime or broadening stored grants. |
| Shared desktop/iOS route manifest | Preserve platform-specific capabilities and routes. Desktop and iOS parity requires native WebView and pairing smoke tests before combining guards. |
| SharedArrayBuffer, VAD and voice IPC changes | Requires cross-origin isolation, native WebView support and measured latency/bandwidth behavior. TinyVoice owns the native pipeline. No unsupported browser feature is assumed. |
| Six total CI workflows and one E2E script | Existing reusable lanes, path filtering, runner pools and artifact reuse do useful separation. Real fork-routing and retry defects are repaired; workflow-file count is not a throughput metric. Release trust boundaries are retained. |
| CI cache/linker improvements | The image already installs sccache and mold; lanes configure cache reuse. Hook area-scoping now avoids unrelated checks without weakening relevant gates. |
| Structured coding handoffs | The prepared host integration supplies a bounded, versioned evidence contract for revisions, selected symbols, changed files and exact validation commands. It does not add unverified agent identities from the roadmap. |
| SVG deduplication and blanket raster/audio conversion | The measured SVGs are not byte-identical copies. Different artwork is retained. Small audio files keep known playback behavior pending platform codec tests. |
| Externalize embedded mascot | Implemented by preserving the exact PNG bytes and importing an emitted asset URL. |
| Font subset loading and sourcemaps | Unicode ranges already exist in `app/src/styles/fonts.css`. The existing sourcemap build policy remains the source of truth; release artifacts need inspection before claiming map removal. |
| Idle animation and route splitting | Decorative mascot/tetrahedron scheduling honors hidden/reduced-motion state; MeshGradient already had those guards. Route splitting stays within the documented root-router boundary. |
| `codegen-units = 1`, new panic behavior and binary reduction targets | The established release profile already has thin LTO, symbol stripping and 16 codegen units. A known project constraint rules out changing to one unit without evidence. No binary-size percentage is claimed. |

## Measured asset and bundle observations

At the audited revision, the 25 chain/token SVG files contain 2,339,683 bytes and no exact duplicate hashes. The largest files embed raster images; their size is not simply redundant vector path data. The three push-to-talk WAVs total 24,828 bytes; `bg.jpg` is 57,007 bytes. These measurements do not support the roadmap's projected 3.5 MB reduction from deleting duplicate files or converting short sounds.

The mascot PNG is 13,102 bytes with SHA-256 `20682bcabdcefe377477072b26386f4eb5f9448eed22089c821dba599788dd17`. Moving its byte-identical content out of the TypeScript module reduces JavaScript payload and parsing work without changing artwork.

The final production frontend build after the frontend fixes reported these sizes. They are useful targets for a future measured loading redesign, not proof of startup speed. Completing the translations adds dictionary bytes; no general payload-reduction claim is made.

| Chunk | Uncompressed kB | Gzip kB |
| --- | ---: | ---: |
| Startup index | 4,071.68 | 1,206.93 |
| I18nContext | 5,703.82 | 1,442.75 |
| Skills | 763.35 | 194.44 |
| FlowCanvasPage | 285.69 | 85.82 |
| MemoryEngineTab | 204.22 | 49.16 |
| Settings | 185.52 | 45.72 |

The build still reports its large-chunk warning. The emitted mascot asset was checked against the original SHA-256 above. Native cold-start, battery drain and end-to-end agent costs have not been measured here.

## Verification of the host fixes

The selected frontend regressions passed **639 tests across 32 files**. They cover the reviewed UX paths, startup and direct hashes, auxiliary windows, IME draft handling, model-provider guards, per-account announcements, managed/manual credential editing, animation activity and locale contracts. This is a selected regression set, not the entire application's Vitest inventory.

The full application TypeScript check, Prettier check, ESLint check and both token gates passed. ESLint retains 67 existing warnings and reported zero errors. Locale checks found no missing or extra keys and no unexpected English fallback values. The production frontend build and its locale/asset checks passed; it is separate from the canonical browser E2E harness.

For users of an earlier PR preview, automatic local provisioning predates the explicit ownership flag within this same PR. Choose **Start local memory** once to reuse the owned container, recover its credential and persist `managed_local`; the named data volume is retained. The migration must not infer ownership from a localhost URL, because a manually operated server can use that same address.

The full required Rust Clippy gate passed with warnings treated as errors, including the desktop application. The default-feature `cargo check -p openhuman --tests` also passed. Both the default-feature and reduced-feature core unit-test builds exceeded the execution container's 8 GiB limit during code generation, even with one Cargo job, no test debug information and incremental compilation disabled. No host Rust unit tests executed in those attempts. Test typechecking is separate evidence from execution; standalone product build and runtime results are recorded separately in the PR description.

The follow-up hosted CI checkpoint `0c66593` passed 27 gate/routing/workflow tests and direct checks against the current default-branch evaluator. The storage-boundary checkpoint `1ca8e2d` passed 17 checker regressions and all four Rust-layout/storage checks with the unchanged 66-site bypass baseline. Its actual native pre-push hook passed root and desktop formatting and strict Clippy. This corrects the earlier broad claim that the initial host tree passed full `rust:layout`: the read-only source-open false positive required the later storage fix.

The complete `pnpm run test:scripts --test-concurrency=1` gate exposed an additional locale-audit module-resolution failure. Loading the app translation graph only when the CLI loads a locale keeps the pure detector importable by native Node. The focused detector suite passed all four cases, and the actual locale CLI reported no unexpected English strings across the 15 translated locales. After the later browser-harness logging regressions were added, the complete script gate reported **836 tests: 833 passed, zero failed, and three platform-specific skips** (two macOS signing cases and one Windows archive-escape case).

The coverage audit then found that `assert-coverage-presence.sh --all` recognized TinyHumans sources as eligible but omitted their directory from both the Git enumeration and the fallback filesystem scan. Both paths now include that layer. Two behavioral regressions failed before the correction and pass afterward; the focused presence suite passes **21/21**. The final complete script gate reports **838 tests: 835 passed, zero failed and the same three platform skips**. The actual scripts/docs-only pre-push hook exits zero with app and Rust checks correctly outside its scope. The CI instructions now point to the active shared lanes and explain that combined changed-line coverage compares the whole PR with its base, rather than with a recent fix checkpoint. No coverage threshold or exclusion was relaxed, and these checker regressions do not establish the unmeasured whole-PR coverage percentage.

The canonical `e2e-web-build.sh` passed its web/typecheck steps and built the full-product standalone core in **9m 36s**. The final frozen `e2e-web-session.sh` run then passed **all nine existing navigation cases in five files**, in **25.2 seconds**, and exited zero. They cover sidebar and command navigation, dirty-draft Stay/Discard, clean canvas exit, direct saved-flow/draft/Settings routes, and Settings history. This selection does not add a browser Save-and-exit claim to the separate component evidence.

The browser was official **Chrome 155.0.8059.84** with Playwright FFmpeg; the pinned Playwright Chromium archives had returned an HTML access-error page. The existing CI installer staged and SHA-verified all ten registry-pinned release modules before the run. Runtime module admission and TLS checks were retained. The harness correction ensures the core's RPC INFO bind-address record survives inherited quiet logging and file constraints, while applying the override only to the launched core. All **35 focused harness regressions** passed. The final source stayed unchanged through the clean browser run and complete script gate; an earlier session whose running shell file was edited is retained as failed evidence and is not the reported pass. The actual pre-push hook for the two harness files passed full app formatting, ESLint, TypeScript, and both token gates; the unchanged Rust inputs were outside that increment's scope.

The final local TinyTools owner checkpoint `f5ce5ae22bfe9fe739ba7d80ce92da026be80b49` (tree `d4ea015cf1125b238c7ec8e2aa2b1826367923eb`) includes two corrections discovered during host composition review. Actual AST patch publication revalidates write access to the original target. Static tool metadata declares the supported minimum permission, allowing read-only policy sessions to admit previews; argument-aware checks still require write permission for absent, false or non-boolean `dry_run` values. Red regressions reproduced both the unauthorized write and the incorrectly denied preview before their fixes.

All required local TinyTools owner gates passed: **1,358 unit/integration tests and nine doctests** with all features, **1,330 tests and nine doctests** with default features, full-workspace formatting, strict Clippy, all-target/all-feature builds, strict rustdoc, the declared Rust **1.88.0** build and `cargo deny --locked check all`. The unchanged **90% per-source-file** coverage gate passed across **145 files**, with a minimum of **90.4594%**. Additional factory/schema, bounded-diagnostic and action-budget tests close the earlier gaps. Every code-sensitive gate was rerun on the final source; the successful vocabulary allowlist result applies because its manifest, lockfile and feature inputs are unchanged. Cargo-deny's duplicate-version and unused-license-allowance warnings are non-fatal, with the policy configuration unchanged.

The local TinyAgents checkpoint `9bd93a5a7ab7468438a24f8a62688e68fe67cfc9` corrects packed-call minimum permissions and observes the actual bound execution target for finish verification. Both bugs were reproduced with red controls. The pack group passed **24/24** tests and the finish-verification group **31/31**, including ten observer cases. The default all-target build then passed in **6m 07s**, followed by **5,174 unit/integration tests and 50 doctests**; 29 unit/integration and seven doc cases retain their existing ignored status. The `basic_graph` example, formatting, both strict all-target Clippy modes, release-version checks and cargo-machete passed. The actual rustdoc broken-link gate passed with 58 existing warnings, none on changed lines; this is not a claim of a warning-free rustdoc build. All-feature, feature-matrix and aggregate-coverage gates remain pending.

An isolated local composition fixture then combined TinyAgents `9bd93a5` with TinyTools `f5ce5ae`, explicitly enabling `tinytools-std/code`. The actual locked, offline runs passed **24 pack tests and 31 finish-verification tests** against that pair. The host fixture's root lockfile added only the 11 expected parser/code packages and the corresponding standard-tool edges, without upgrades or removals; its storage-driver graph remains on 0.11.0. Both parent-index gitlinks remained unchanged. These results establish compatibility for those owner tests, not execution of the six host composition cases.

The execution service disconnected before the host composition launch returned a session ID or test result. The browser environment also reported offline. The remaining TinyAgents matrix/coverage and host native/combined-coverage work are blocked by that outage; no host test pass or surviving process is inferred. Resume by inspecting the preserved run receipts before starting another compiler.

The owner changes preserve the host's compatible dependency baseline. TinyTools must land in its owning repository before a consuming TinyAgents pointer commit. An isolated local fixture may validate the prepared owner source with unchanged parent-index gitlinks; that evidence does not complete publication or canonical recursive-fetch checks. OpenHuman must consume the delivered owner chain before its new registry integration can be validated as a product feature.

## Parser experiment

The reproducible TinyTools `ast_benchmark` example was run in release mode on
x86_64 Linux with Rust 1.96.1, 20 samples and 128 synthetic functions per file.
These medians include hashing, parsing, queries, diagnostics and folded-source
construction, but exclude JSON serialization. They are local measurements under the current machine
load and are not product-wide latency commitments.

| Language | Cold analysis | Warm full parse | Cached analysis | Incremental analysis | Source / skeleton bytes |
| --- | ---: | ---: | ---: | ---: | ---: |
| Rust | 6.62 ms | 2.12 ms | 0.80 ms | 0.94 ms | 15,012 / 11,812 |
| TypeScript | 6.45 ms | 3.87 ms | 0.94 ms | 1.07 ms | 18,340 / 13,860 |
| Python | 3.86 ms | 2.53 ms | 0.84 ms | 0.90 ms | 15,268 / 12,452 |

The small synthetic bodies produce modest skeleton savings. A full serialized
analysis with all 128 symbol records is about 70–76 kB and can exceed the raw
source size. This is why the native adapter pages declarations and text rather
than returning an unbounded analysis record. Token savings must be measured on
actual tasks, with the selected tokenizer, before setting an acceptance target.

## Integration and release sequence

1. Validate the pure parser/template crate and filesystem adapter in TinyTools, including all enabled features.
2. Publish the owning TinyTools contribution, open its upstream PR and land it there, as its repository guidelines require before a consuming pointer change.
3. Update TinyAgents' single nested TinyTools gitlink in a separate commit, validate that graph and publish its own contribution.
4. After the required owner change is available, verify a fresh canonical recursive checkout, update OpenHuman's TinyAgents gitlink and enable the native adapter. Do not copy generic parser/template implementation into the host to bypass ownership.
5. Run host registry/policy tests and native platform smoke checks. Keep release/merge decisions behind the repository's normal CI gates.

This review does not treat a successful web build as a desktop/iOS runtime test, a syntax test as a type-check, or a prepared upstream change as a published application capability. Concrete verification results and any environmental blockers belong in the PR description alongside the commit that was actually tested.
