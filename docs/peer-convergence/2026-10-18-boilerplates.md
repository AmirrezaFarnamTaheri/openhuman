# Boilerplate repository absorption audit

Snapshot: 2026-10-18 UTC. This is static source review of four named repositories plus a pinned follow-up of all ten repositories in the bounded search sample. No external repository code was imported, executed, or installed.

## Recommendation

Prioritize three ideas: schema-driven chunk inspection and parameter forms; testing generated combinations as actual sources; and a desktop smoke test that verifies the renderer-to-host seam. Keep framework-specific deployment, authentication and state-management code in separate optional template packs after their prerequisites and ownership are specified. The current TinyTools engine stays a pure syntax/rendering library, with filesystem publication and policy in its native adapter.

Effort estimates below are engineering estimates for a focused implementation and tests by a developer familiar with OpenHuman. They exclude dependency upgrades and complete product rollout.

## Inspiration beyond importing a starter project

The reviewed projects are useful even when their framework is old, their language is unsupported by the current parser, or their source cannot be copied. A generator can inspire an output contract; a README workflow can inspire native UI; a teaching fixture can inspire an acceptance test. Direct implementation/text reuse and independent design are separate decisions. The table below states derived native designs; it does not claim these richer features already exist in the peers or OpenHuman.

| Source                       | Small idea                                                         | Independent native feature                                                    | Broader composition                                                                                     |
| ---------------------------- | ------------------------------------------------------------------ | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| react-boilerplate            | Source and its test form one intended output set                   | A versioned paired generation plan with an explicit assertion requirement     | A reusable-code studio tracks preview, publication and subsequent behavioral verification               |
| electron-react-boilerplate   | Desktop readiness includes a working renderer/host exchange        | A native Tauri smoke contract with event cleanup and failure receipts         | Release validation connects launch, bridge, cancellation and teardown evidence                          |
| ChristianLempa/boilerplates  | Parameter metadata is part of usability and validation             | Typed grouped fields, conditional visibility and bounded variants             | A chunk inspector combines forms, source previews and declared dependency requirements                  |
| aspnetcorehero/Boilerplate   | Permissions and audit events should be visible concepts            | UI affordances derived from the host's action model                           | User-visible availability, admission and actual outcome share an understandable vocabulary              |
| ASP.NET Boilerplate          | Audit selection can be explicit instead of indiscriminate          | A bounded policy for which host actions emit which audit fields               | Evidence capture and privacy/retention choices become inspectable                                       |
| Next.js Boilerplate          | Test server startup belongs in the test contract                   | Deterministic fixture readiness and teardown                                  | Browser validation can distinguish unavailable dependencies from failed product behavior                |
| Node Express boilerplate     | Validation and error conversion need defined order                 | Schema admission and stable public error categories                           | Generated service patterns share meaningful failure behavior with host tooling                          |
| H5BP Nginx                   | A configuration is an include graph with operational prerequisites | A separately scoped configuration preview and validation plan                 | Deployment templates can expose certificate, routing and caching assumptions before application         |
| HTML Hello                   | A minimal preview should make local content easy to inspect        | An independently designed bounded local preview capability                    | Artifact selection, rendering and explicit host serving policy can support a general preview workflow   |
| freeCodeCamp Express         | Small observable contracts support understandable exercises        | A teaching/test fixture with exact request/response expectations              | A pattern library can explain why generated code fails a particular requirement                         |
| Fiber boilerplate            | A compact route module can establish conventions                   | A supported-language service pattern with explicit route/error/context seams  | Multiple patterns can compose into a testable starter without importing a second host server            |
| WordPress Plugin Boilerplate | Registration and cleanup should name what they own                 | A generic lifecycle contract for callbacks and resources                      | Plugin or event integrations can test idempotence, teardown and ownership independent of PHP            |
| SaaS Boilerplate             | Account setup and teardown are part of test isolation              | Tenant-scoped test fixtures with explicit unavailable states                  | Authentication and onboarding tests can share reproducible lifecycles without personal-account fallback |
| GenLayer project boilerplate | Stochastic outputs need a precise accepted-result contract         | A parser/validator tested against the production path and adversarial outputs | Agent-generated results can be compared under declared evidence and uncertainty criteria                |

These compositions require new specifications and tests, not a bulk copy of the source library. Unsupported grammars remain a concrete implementation constraint on structural rendering; they do not make UI, lifecycle or validation ideas unusable. Missing or restrictive licenses block proposed source/text redistribution while leaving independently specified mechanisms available for further design.

## Fit against the published structural-code contract

The proposed destination is the published
[TinyTools code adapter](https://github.com/tinyhumansai/tinytools/blob/daec6c7a563336dd83d2e65ae38b2b169179aada/crates/tinytools-std/src/code/README.md),
consumed through the separate TinyAgents fork composition. Its current twelve
patterns are single-file, function-body or member chunks. Search returns
bounded bundled metadata; custom patterns require an explicit gated
`manifest_path`. Inspection binds the manifest/template contents to a
`chunk_revision`, and application returns a bounded source page plus revision
and publication receipt. It does not install dependencies or execute generated
source.

That baseline creates precise extension points:

| Scale             | Concrete addition                                                                   | Boundary it must preserve                                                                                   |
| ----------------- | ----------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- |
| Tiny detail       | Show type, required/default status and content revision in a parameter inspector    | A form value is data; the host remains authoritative for destination and write permission                   |
| Primitive         | Deterministic bounded variant enumeration                                           | A threshold for changing strategy is not an absolute resource budget                                        |
| Single feature    | Preview and apply one supported source pattern with a meaningful behavioral fixture | Syntax success does not prove imports, types or runtime behavior                                            |
| Template pack     | A small versioned family with prerequisites, licenses and per-pattern tests         | New dependencies are declared, never installed as a side effect of rendering                                |
| Module            | Multi-file generation plan and explicit partial-failure/rollback contract           | Per-file atomic rename does not imply atomic publication of an entire project                               |
| Whole application | An optional separately maintained starter project                                   | Its framework, auth, database, updater and deployment lifecycle cannot become hidden TinyTools dependencies |

The current grammar set includes Rust, TypeScript/TSX, JavaScript, Python, Go,
JSON and TOML. C#, PHP, raw HTML and Nginx configuration are outside that set.
A pattern in one of those languages needs an owner-supported grammar or a
separate appropriately validated generator; labeling its text as another
language would bypass the existing syntax contract.

## Exact source snapshots

| Repository                                            | Default-branch SHA                         | API last push | Top-level license                                                                                                                     |
| ----------------------------------------------------- | ------------------------------------------ | ------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| react-boilerplate/react-boilerplate                   | `d19099afeff64ecfb09133c06c1cb18c0d40887e` | 2023-03-23    | [MIT](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/LICENSE.md)                |
| electron-react-boilerplate/electron-react-boilerplate | `e5b8242d8b21a4290c3a1f18ae3b64167e9de04a` | 2026-10-12    | [MIT](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/LICENSE) |
| ChristianLempa/boilerplates                           | `02d85a2d55c0e455d4af9b5af00984430e4fe404` | 2026-08-31    | [MIT](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/LICENSE)                           |
| aspnetcorehero/Boilerplate                            | `c5c9bcab020d2b18afcdd31e6345e465f0d5358a` | 2023-07-04    | [MIT](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/LICENSE)                            |

Metadata came from each repository's GitHub REST resource; exact heads came from `git/ref/heads/{default_branch}`. All four reported `archived=false`. Last-push dates describe repository activity, not a security or maintenance guarantee. Recursive trees were returned with `truncated=false`; this audit still reads selected files rather than claiming review of every file.

OpenHuman and TinyTools currently declare `GPL-3.0-only` in their workspace manifests. The four inspected MIT licenses permit copying, modification and redistribution subject to preserving the copyright and permission notices. This makes their own MIT-licensed source a plausible reuse candidate in the GPL project; this is a source-scope assessment, not clearance of every dependency, logo, bundled asset or downstream generated project. Preserve exact upstream attribution for any copied material. No attribution obligation is triggered by this audit because it imports no source.

## Concrete candidates

| Candidate                                                                   | Evidence                                                                                                                                                                                                                                                                                                                                                                                                                                                                                     | Owning destination                                         | Work and acceptance                                                                                                                                                                                                                                                                                           | Estimate / disposition                                               |
| --------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------- |
| Generate a component together with its behavioral test                      | [component generator](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/index.js); [test template](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/test.js.hbs)                                                                                                                                                        | TinyTools template pack, with OpenHuman preview            | A TSX pattern plus required assertion slot, explicit destination paths and preview. Reject duplicate destinations and missing assertions. The upstream test contains a deliberately failing placeholder and skipped snapshot: write a real behavior test for OpenHuman instead of copying those placeholders. | 1–2 days; useful after multi-file application semantics are designed |
| Validate all supported generated variants                                   | [generation/lint driver](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/scripts/generate-templates-for-linting.js); [dependency matrix](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/validation/dependency_matrix.py); [matrix tests](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/tests/test_dependency_matrix.py) | TinyTools fixture/CI tooling                               | Render every built-in default and bounded relevant parameter combinations, then parse and, where a package fixture exists, typecheck/test. Keep a fixed cap, identify sampled cases and refuse claims of exhaustive combination coverage. Do not execute arbitrary template-specified commands.               | 1–2 days; strong near-term improvement                               |
| Parameter inspector with grouped, conditional fields                        | [typed variables](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/template/variable.py); [workflow documentation](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/README.md)                                                                                                                                                                                                              | OpenHuman UI; typed metadata extensions owned by TinyTools | Display declared type, default, description and required status before applying. Future enum/range/group/dependency metadata needs a versioned schema and deterministic validation. Start with existing six parameter kinds. Preview must display the exact revision and destination.                         | 2–4 days; high product value                                         |
| Secret-reference parameter type                                             | [secret metadata](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/template/variable.py)                                                                                                                                                                                                                                                                                                                                                | OpenHuman credential boundary, then owner schema           | Carry a secret reference rather than embedding plaintext defaults or logging values. Upstream rejects defaults for autogenerated secrets. Define redacted inspect/preview behavior and host resolution before adding this kind.                                                                               | 2–4 days; requires a separate accepted contract                      |
| Desktop startup and bridge smoke test                                       | [desktop smoke runner](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.erb/scripts/smoke-test.cjs); [OS matrix](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.github/workflows/test.yml)                                                                                                                                                   | OpenHuman app CI                                           | Assert the main route, loaded assets, no runtime exceptions, and a bounded renderer-to-host round trip. Translate the checks to Tauri's existing runtime/test seam. Keep deadlines and process cleanup. The upstream uses Electron CDP and IPC; its exact runner is platform-specific.                        | 2–3 days; strongest build-system idea                                |
| Typed subscription API with explicit cleanup                                | [preload bridge](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/src/main/preload.ts)                                                                                                                                                                                                                                                                                                                                 | OpenHuman frontend host-event adapters                     | Return a disposer from subscriptions, constrain event names, test unmount cleanup and duplicate subscriptions. The source is a useful API pattern; Tauri must continue using its own command/event authorization.                                                                                             | 0.5–1.5 days; use when an actual cleanup gap is found                |
| Release-stage verification across operating systems                         | [workflow](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.github/workflows/test.yml); [build configuration](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/package.json)                                                                                                                                                                    | OpenHuman release CI                                       | Separate source checks from package construction and runtime smoke. Attach exact artifact and source identities to receipts. Map macOS signing and Linux display requirements to existing Tauri workflow rather than adding a second desktop runtime.                                                         | 2–4 days; compare existing release gates first                       |
| A permission/audit inspection panel                                         | [permission table](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Areas/Admin/Views/Permission/Index.cshtml); [permission names](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Application/Constants/Permissions.cs)                                                                                                                    | OpenHuman settings / run receipts                          | Group capabilities and show effective decisions and provenance. Preserve OpenHuman's actual dynamic tool policy, including dry-run classification. Do not translate CRUD role claims into tool authorization by assumption.                                                                                   | 2–4 days; presentation idea only                                     |
| Explicit application/domain/infrastructure boundaries in generated services | [architecture](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/README.md); [authorization implementation](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Permission/PermissionAuthorizationHandler.cs)                                                                                                                                                               | Optional service template pack                             | Produce small tested Rust/TypeScript equivalents only when requested. The existing C# application and ASP.NET authorization stack do not belong inside TinyTools or the Rust host. C# is outside the current grammar set.                                                                                     | 3–5 days per reviewed service pattern; later                         |

## Maturity and fit of each repository

### React Boilerplate

The pinned package is version 4.0.0 and declares React 16.8.6, Redux 4.0.1, Redux-Saga 1.0.2 and a Webpack-based build. Its generator can choose memoization, translated messages and asynchronous loading, and generates paired test files. These are concrete template-design ideas. Its dependencies and test scaffolding require modernization before execution in the current host; the audit did not run them. The useful unit of reuse is a reviewed generated pattern and its assertions. [Package evidence](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/package.json); [generator evidence](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/index.js).

### Electron React Boilerplate

The current snapshot uses Electron/Vite, TypeScript, React 19.3 and a three-OS CI matrix. The smoke runner verifies a rendered page, image, links, absence of a Vite error overlay, an IPC round trip and absence of renderer exceptions. This is a concrete example of testing the shipped application seam. OpenHuman uses Tauri 2, so the most valuable transfer is the verification structure and typed cleanup API. Introducing Electron, its updater or its packaging tree would create a new runtime to own. [Package](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/package.json); [smoke checks](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.erb/scripts/smoke-test.cjs).

### Christian Lempa Boilerplates

This is the closest match to a discover/inspect/configure/render product flow. Its variable model includes descriptions, types, defaults, options, ranges, dependency conditions and special secret handling. Its matrix builder chooses exhaustive or reduced cases using a configurable threshold and has dedicated tests; that threshold is not a strict final-output cap, as detailed below.

There is a material migration boundary at the inspected SHA: the README says version 0.2.0 uses `template.json` and `files/`, and legacy YAML manifests and .j2 files are unsupported. The same tree still contains old `library/**/template.yaml` entries, including a PostgreSQL entry marked draft. The README now points to a separate template-library repository. Therefore bulk ingestion of this repository's legacy library would be unsound. A future importer must pin the supported format, inspect the separate library, and validate each selected template. [Format contract](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/README.md); [legacy draft example](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/library/compose/postgres/template.yaml); [new-format fixture](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/tests/test_dependency_matrix.py).

### ASP.NET Core Hero

The README targets ASP.NET Core 5 / Entity Framework Core 5 and advertises MVC/WebAPI, role-claim permissions, audit activity, localization, RTL and dark-mode UI. The inspected permission handler checks claims with a fixed local issuer; this is a web-application authorization design, not a substitute for dynamic tool admission. Its permission table and boundary organization can inform OpenHuman UI and future service patterns. No framework, authentication or database implementation should move into the syntax engine. [README](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/README.md); [handler](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Permission/PermissionAuthorizationHandler.cs).

## Detailed adaptation and failure analysis

### React Boilerplate: paired files need a publication plan

The component generator creates source and test files, conditionally adds
messages and a loadable wrapper, and then runs formatting. Each action sets
`abortOnFail`; that stops later actions but does not by itself establish a
transaction that rolls back earlier files. Its test template deliberately
contains a failing placeholder and a skipped snapshot. Those are scaffolding
prompts, not a completed regression suite. The useful transfer is “a pattern
includes the behavior that must be tested,” not importing an old green-looking
test shell. See the pinned [generator](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/index.js)
and [test template](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/test.js.hbs).

A first TinyTools addition can remain single-file: generate a component with
explicit properties, accessible name and a required action callback, then
apply a separate test pattern. A later bundle format should declare every
destination, existing-file revision, create/replace mode, dependency and output
digest before writing. The host can present that plan and authorize the exact
destinations. A portable multi-file operation must state whether it provides
all-or-nothing publication, a recoverable journal, or explicit partial success.
It must not reuse the single-file `written: true` field as proof that an entire
bundle committed.

Proposed tests:

- Duplicate paths, case-colliding destinations and traversal fail before the
  first write; path comparison follows the target filesystem's behavior.
- Source creation succeeds but a test destination becomes stale: the receipt
  reports the exact published/untouched files and supports the documented
  recovery policy.
- A generated test must fail against the deliberately incorrect behavior and
  pass against the intended behavior; a snapshot placeholder is insufficient.
- The optional i18n branch uses OpenHuman's existing `useT()` and locale
  contracts. A generated loadable wrapper cannot violate the route-only lazy
  import exception.
- Removing a parameter or switching a branch cannot silently leave an obsolete
  generated import or overwrite a later hand edit.

The generator's old React/Redux/Webpack dependencies are reasons to author
current, narrow templates. They are not a reason to change OpenHuman's router,
state layer or build system.

### Christian Lempa: inspectable variables and a strictly bounded matrix

The variable model keeps explicit/default origins, validates types, supports
sections and conditions, and separates secret-generation configuration from
ordinary values. Its validator rejects defaults on autogenerated secrets.
The inspected implementation also accepts broader string conversions and
legacy configuration forms than TinyTools' current typed JSON schema. A
frontend should not silently add those coercions to `dry_run` or other native
permission-sensitive arguments. [Variable implementation](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/template/variable.py)

A useful first inspector needs no new schema: render the existing six parameter
kinds, required status, descriptions, slot restrictions, declared dependencies,
destination and current `chunk_revision`. Preserve omission versus an explicit
false/zero/empty value. Show validation next to the field, but let the owner
renderer make the authoritative decision. Enum/range/group/conditional fields
would be a versioned schema extension with one owner evaluator, not a second
JavaScript interpretation of template expressions.

The matrix implementation collects boolean toggles and condition values,
enumerates a Cartesian product below `max_combinations`, otherwise emits
defaults, broad boolean cases and individual branch overrides, then deduplicates
effective satisfied states. This is practical branch sampling. The reduced path
does **not** clamp its number of emitted cases to `max_combinations`; enough
variables/values can still exceed that threshold. It also parses only explicit
equality/inequality conditions in the inspected helper, so “reduced” is not a
claim to cover every interaction. A TinyTools version should check the final
case count and serialized fixture budget and report total/selected/skipped
counts and its selection strategy. [Matrix source](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/validation/dependency_matrix.py)

| Failure case                                                | Proposed acceptance boundary                                                                                      |
| ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| Fifty branch variables exceed the selected budget           | Stop at the absolute case/byte cap, retain deterministic ordering and disclose omitted combinations               |
| Disabled required field retains an earlier value            | One owner evaluator decides its effective state; hidden stale values cannot change rendered output unexpectedly   |
| Condition names a missing variable or forms a cycle         | Refuse the manifest with a bounded diagnostic before showing an actionable form                                   |
| String `"false"` is supplied to a boolean                   | Follow the versioned parameter schema; do not reuse permissive CLI coercion for native permission flags           |
| Custom manifest changes after inspection                    | Reject stale `chunk_revision` and ask for a fresh preview, even if the normalized metadata looks equivalent       |
| Secret value appears in a default, preview or test artifact | A future secret-reference kind remains redacted and host-resolved; generic rendering must not fetch credentials   |
| Template comes from the old YAML/.j2 library                | Require explicit format migration and per-pattern fixtures; do not infer compatibility from repository membership |

Secret-reference support is a separate host contract. The safest useful initial
behavior is to generate a documented environment-variable reference, not a
credential value. Any later resolver must define storage scope, redaction,
consent, audit and whether secrets may enter a generated file at all. The MIT
license on the generator does not settle those runtime decisions or the
separate template library's contents.

### Electron React Boilerplate: test the actual seam without importing Electron

The smoke runner starts `npm start`, locates the renderer over CDP, checks
visible content and an image, performs a preload IPC round trip, and checks
renderer exceptions. It has request deadlines and process-tree cleanup. This
is useful evidence design, but it is a development-server smoke, not proof that
a signed packaged artifact works. Its fixed ports, accumulated unbounded output
string and Linux CI `--noSandbox` switch should not be copied into OpenHuman.
[Exact smoke implementation](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.erb/scripts/smoke-test.cjs)

OpenHuman uses Wry/Tauri, whose desktop WebView is not an Electron CDP target.
Keep two distinct tests: the existing browser SPA/core path for route and RPC
behavior, and native built-app smoke for each supported WebView/asset protocol.
The native smoke should assert the selected artifact identity, successful core
handshake, one actual authorized RPC round trip, loaded compiled assets and
clean shutdown. A mocked `invoke` or a Vite page alone cannot prove that seam.

The peer's `on()` returns a disposer bound to the exact listener wrapper.
OpenHuman already uses that shape in
[`useRunQueueEvents`](../../app/src/features/conversations/aui/useRunQueueEvents.ts)
and [follow-up suggestion events](../../app/src/features/conversations/aui/useFollowupSuggestionEvents.ts):
their effects return the service subscription's cleanup. The productive next
step is a targeted lifecycle regression where a leak exists, not adding a new
event abstraction. The peer's `once()` helper does not expose an early disposer;
a component may still need cancellation if the event never arrives.
[Preload bridge](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/src/main/preload.ts)

Required future checks include StrictMode subscribe/cleanup/resubscribe, account
switches, unmount before a one-shot result, duplicate reply IDs, an early child
process exit, an occupied port, a failed handshake, and a shutdown timeout.
Bound stdout/stderr capture and redact credentials. A desktop test should
exercise the supported production sandbox and report the real failure instead
of disabling the boundary to make startup succeed.

### ASP.NET Core Hero: expose effective authorization rather than a role analogy

The permission handler checks a named claim and a fixed local issuer before
succeeding. Its administration table is a useful presentation example for
grouped permission names. That is a different contract from an OpenHuman tool
call whose effective permission depends on arguments, trusted workspace,
autonomy, approval and selected packed target. A CRUD claim cannot authorize a
structural write merely because both are labeled “edit.”
[Handler](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Permission/PermissionAuthorizationHandler.cs)

A useful product panel would display the effective capability, the source of
the decision, the current workspace and whether the entry reflects a request,
an admitted attempt or a completed result. For `use_skill`, show the selected
target rather than assigning every call the wrapper's minimum discovery
permission. A denied call should remain visible as a denial and should not
appear in the executed set. This uses the existing TinyAgents observer and
OpenHuman policy; no ASP.NET authorization service is required.

Proposed acceptance cases are a read-only `ast_patch` preview, a real write
requiring approval, an unknown packed target, policy revocation before
publication, account/workspace switching, and a failed started attempt. The
display must retain these distinctions and must not allow its local UI state
to override backend enforcement. C# service templates require a separate
supported-language decision and framework-specific tests before inclusion.

## Requested search snapshot and pinned follow-up

Requested URL: https://github.com/search?q=boilerplate&type=repositories&p=2

The request specifies query `boilerplate`, repository search, page 2 and no
explicit sort. The GitHub connector rejected the HTML search URL and the
initial web retrieval could not open it. The original connector API sample
therefore used `query=boilerplate,page=2,per_page=10`, leaving sort unspecified.
Its membership is retained below. It is not claimed to reproduce the browser
page's exact ordering, which can change independently.

The deeper follow-up resolved all ten sampled repositories to exact default-
branch revisions, read their root inventory/README and available license or
package metadata, and inspected the concrete implementation slices linked
below. This upgrades the sample from identity-only discovery to a **bounded
source assessment**. It is not a full security audit or a reproduced build.

| Repository and exact inspected revision                                                                                                                                                              | Implementation evidence and concrete fit                                                                                                                                                                                                                                                                                                                                                                                                | Reuse screen                                                                                                                                                                                                                                                                                                                                                 |
| ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| [aspnetboilerplate/aspnetboilerplate@`21f57d282f542aa4221fc5cd5a4ac76346d8f5c7`](https://github.com/aspnetboilerplate/aspnetboilerplate/commit/21f57d282f542aa4221fc5cd5a4ac76346d8f5c7)             | [AuditingHelper](https://github.com/aspnetboilerplate/aspnetboilerplate/blob/21f57d282f542aa4221fc5cd5a4ac76346d8f5c7/src/Abp/Auditing/AuditingHelper.cs) separates selection, record creation/serialization and storage under a suppressed surrounding transaction; useful audit-lifecycle test questions, not a replacement .NET host                                                                                                 | [MIT, Volosoft](https://github.com/aspnetboilerplate/aspnetboilerplate/blob/21f57d282f542aa4221fc5cd5a4ac76346d8f5c7/LICENSE.md); retain notices, review selected dependencies                                                                                                                                                                               |
| [ixartz/Next-js-Boilerplate@`b83157a8018f55b926a3c25a4b7a7e14b0dfbc35`](https://github.com/ixartz/Next-js-Boilerplate/commit/b83157a8018f55b926a3c25a4b7a7e14b0dfbc35)                               | [Playwright configuration](https://github.com/ixartz/Next-js-Boilerplate/blob/b83157a8018f55b926a3c25a4b7a7e14b0dfbc35/playwright.config.ts) binds server URL/port, owns a test database process, forbids focused tests in CI and runs Firefox there; useful test-harness ownership, not a Next.js migration                                                                                                                            | [MIT, Remi W.](https://github.com/ixartz/Next-js-Boilerplate/blob/b83157a8018f55b926a3c25a4b7a7e14b0dfbc35/LICENSE); service/dependency licenses remain separate                                                                                                                                                                                             |
| [hagopj13/node-express-boilerplate@`179ae84efec61b14206d0305d941daed6c6d07f9`](https://github.com/hagopj13/node-express-boilerplate/commit/179ae84efec61b14206d0305d941daed6c6d07f9)                 | [Validation middleware](https://github.com/hagopj13/node-express-boilerplate/blob/179ae84efec61b14206d0305d941daed6c6d07f9/src/middlewares/validate.js) validates selected request sections and forwards normalized values; [error handling](https://github.com/hagopj13/node-express-boilerplate/blob/179ae84efec61b14206d0305d941daed6c6d07f9/src/middlewares/error.js) separates operational/public errors from production internals | [MIT, Hagop Jamkojian](https://github.com/hagopj13/node-express-boilerplate/blob/179ae84efec61b14206d0305d941daed6c6d07f9/LICENSE); generator fixtures need current package verification                                                                                                                                                                     |
| [h5bp/server-configs-nginx@`d2f2c3e2fac76f429adb738894499b8fb756d2f3`](https://github.com/h5bp/server-configs-nginx/commit/d2f2c3e2fac76f429adb738894499b8fb756d2f3)                                 | [Composed configuration](https://github.com/h5bp/server-configs-nginx/blob/d2f2c3e2fac76f429adb738894499b8fb756d2f3/nginx.conf) separates MIME/security/performance includes; [TLS virtual-host fixture](https://github.com/h5bp/server-configs-nginx/blob/d2f2c3e2fac76f429adb738894499b8fb756d2f3/test/vhosts/secure.server.localhost.conf) makes environment assumptions explicit                                                    | [MIT-style H5BP notice](https://github.com/h5bp/server-configs-nginx/blob/d2f2c3e2fac76f429adb738894499b8fb756d2f3/LICENSE.txt); Nginx is outside the current syntax engine                                                                                                                                                                                  |
| [4GeeksAcademy/html-hello@`a9b4fe4a305a1521f1ad59a0d8ec897b2634816a`](https://github.com/4GeeksAcademy/html-hello/commit/a9b4fe4a305a1521f1ad59a0d8ec897b2634816a)                                   | [Development server](https://github.com/4GeeksAcademy/html-hello/blob/a9b4fe4a305a1521f1ad59a0d8ec897b2634816a/server.py) can execute `app.py` on a GET and binds a Flask debug server on all interfaces; unsuitable as a passive generated-preview host                                                                                                                                                                                | No explicit reuse grant was established in the inspected root, README or server; use independent design only                                                                                                                                                                                                                                                 |
| [freeCodeCamp/boilerplate-express@`4a635b463dcdeb2eb0104d474ca455b385b7a43d`](https://github.com/freeCodeCamp/boilerplate-express/commit/4a635b463dcdeb2eb0104d474ca455b385b7a43d)                   | [Server](https://github.com/freeCodeCamp/boilerplate-express/blob/4a635b463dcdeb2eb0104d474ca455b385b7a43d/server.js) delegates to a curriculum grading helper and configurable cross-origin behavior; it is a teaching harness, not a production API template                                                                                                                                                                          | No license file or declaration was present in the inspected root/package; the parent organization's reputation is not a source license                                                                                                                                                                                                                       |
| [gofiber/boilerplate@`a18afd809b7a4fc3fbb82142d5c14ad93fc7a8e9`](https://github.com/gofiber/boilerplate/commit/a18afd809b7a4fc3fbb82142d5c14ad93fc7a8e9)                                             | [Entry point](https://github.com/gofiber/boilerplate/blob/a18afd809b7a4fc3fbb82142d5c14ad93fc7a8e9/app.go) wires recovery/logging, database, versioned routes and static files; an optional Go service pattern after dependency and shutdown review                                                                                                                                                                                     | [MIT, Fiber](https://github.com/gofiber/boilerplate/blob/a18afd809b7a4fc3fbb82142d5c14ad93fc7a8e9/LICENSE); [go.mod](https://github.com/gofiber/boilerplate/blob/a18afd809b7a4fc3fbb82142d5c14ad93fc7a8e9/go.mod) pins Go 1.26 and Fiber v3, distinct from a generic standard-library handler                                                                |
| [DevinVinson/WordPress-Plugin-Boilerplate@`a8291082b5fd23bb84541e7c4089feaa1e99e597`](https://github.com/DevinVinson/WordPress-Plugin-Boilerplate/commit/a8291082b5fd23bb84541e7c4089feaa1e99e597)   | [Hook loader](https://github.com/DevinVinson/WordPress-Plugin-Boilerplate/blob/a8291082b5fd23bb84541e7c4089feaa1e99e597/plugin-name/includes/class-plugin-name-loader.php) stores callback/priority/argument-count registrations separately from activation; useful lifecycle vocabulary, not a PHP runtime addition                                                                                                                    | [README](https://github.com/DevinVinson/WordPress-Plugin-Boilerplate/blob/a8291082b5fd23bb84541e7c4089feaa1e99e597/README.md) specifies GPL v2 or later; preserve that grant and [bundled GPL text](https://github.com/DevinVinson/WordPress-Plugin-Boilerplate/blob/a8291082b5fd23bb84541e7c4089feaa1e99e597/plugin-name/LICENSE.txt), never relabel it MIT |
| [ixartz/SaaS-Boilerplate@`e3952a7ed5b0ef172ac4363c4644b8c334d1094b`](https://github.com/ixartz/SaaS-Boilerplate/commit/e3952a7ed5b0ef172ac4363c4644b8c334d1094b)                                     | [Playwright lifecycle](https://github.com/ixartz/SaaS-Boilerplate/blob/e3952a7ed5b0ef172ac4363c4644b8c334d1094b/playwright.config.ts) declares setup/teardown dependencies for isolated accounts; comments point to separate starter offerings for those implementations                                                                                                                                                                | [MIT, Remi W.](https://github.com/ixartz/SaaS-Boilerplate/blob/e3952a7ed5b0ef172ac4363c4644b8c334d1094b/LICENSE) covers inspected public files, not promised commercial setup code                                                                                                                                                                           |
| [genlayerlabs/genlayer-project-boilerplate@`e685f1f12c4c357787d48390692a654baf576f03`](https://github.com/genlayerlabs/genlayer-project-boilerplate/commit/e685f1f12c4c357787d48390692a654baf576f03) | [Direct pattern tests](https://github.com/genlayerlabs/genlayer-project-boilerplate/blob/e685f1f12c4c357787d48390692a654baf576f03/tests/direct/test_patterns.py) separate deterministic helpers from mocked VM/LLM contract tests; useful testing tiers, not evidence of financial correctness or live consensus                                                                                                                        | [MIT, YeagerAI](https://github.com/genlayerlabs/genlayer-project-boilerplate/blob/e685f1f12c4c357787d48390692a654baf576f03/LICENSE); GenVM/Studio and external services require separate review                                                                                                                                                              |

### What each sample would have to prove

The following are proposed tests for a selected port or pattern. None was run
against the peer during this investigation.

| Sample               | First useful unit and proposed acceptance/failure cases                                                                                                                                                                                                                                                                           |
| -------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| ASP.NET Boilerplate  | A typed audit-selection contract, not the framework: an explicitly excluded field is absent; a rolled-back business operation does not erase its failed-attempt record; serializer failure produces an explicit diagnostic rather than an apparently empty successful capture                                                     |
| Next.js Boilerplate  | Test-server lifecycle pattern: the configured base URL matches the actual bound port; a failed migration prevents the UI suite from running against stale state; server cleanup runs after a failed browser assertion                                                                                                             |
| Node/Express         | Small validation/error template: invalid body/query/params return the intended status; normalized values alone reach the handler; unknown internal errors do not expose stack or secrets in production. The inspected status expression also needs a regression for a supplied non-400 status before reuse                        |
| H5BP Nginx           | A reviewed configuration pack: `nginx -t` validates the complete include tree; missing certificate/include files fail explicitly; headers, redirects and static caching are tested on the intended deployment. Do not transplant fixed connection/file-limit values without platform capacity checks                              |
| HTML Hello           | Independently specified preview host, with direct source reuse separately blocked pending a license grant: requesting a page never executes arbitrary project Python; binding defaults to the authorized local interface; traversal and missing-file diagnostics are bounded and nonexecuting                                     |
| freeCodeCamp Express | Teaching fixture only: grading dependencies are pinned and isolated; origins follow the consuming project's policy; disabling a teaching restriction cannot disable the host's bearer/origin boundary. No production starter adoption is recommended                                                                              |
| GoFiber              | Optional Go pattern: constructor/database failure is surfaced; shutdown drains or cancels in-flight work; validation fails before a write; prefork is an explicit platform option with its own tests                                                                                                                              |
| WordPress            | Hook-registration design only until PHP support is scoped: callback arity/priority are retained; repeated activation follows an explicit idempotency contract; deactivation removes owned resources without changing unrelated hooks                                                                                              |
| SaaS Boilerplate     | Account-fixture lifecycle: each test uses its own tenant/account; teardown follows a failed test; missing external credentials or absent setup files produce an explicit unavailable result rather than reusing a personal account                                                                                                |
| GenLayer             | Deterministic-versus-integration test separation: malformed/multiple JSON objects are rejected according to a strict parser policy; a model refusal cannot satisfy a success criterion; a changed unvalidated field is reported as outside the assertion's coverage; mocked VM success is never labeled live-network verification |

Two sample findings materially affect reuse. The Node error converter's
`error.statusCode || error instanceof mongoose.Error ? ...` expression selects
the conditional's 400 branch whenever the left side is truthy; a supplied
non-400 status therefore needs an intentional policy and a regression before
copying. The GenLayer test file defines its own `_parse_llm_json` helper and
uses broad `Exception` acceptance in one failure assertion. Those tests are
useful examples but do not prove the production helper or exact failure cause.
An OpenHuman contribution should exercise the real owner implementation, which
is the same principle used by this PR's host composition targets.

## Implementation boundary

The current TinyTools and TinyAgents owner source is published with passing matrices. OpenHuman's host integration and application gates remain the next delivery boundary. Then propose a separate, versioned expansion for multi-file bundles, typed form metadata and bounded variant testing. Every future imported pattern needs an upstream source SHA/license record, a supported language, typed inputs, declared destination, meaningful fixtures, and explicit host publication policy. Existing source revision checks, one-pass expansion, preview classification and filesystem gates remain authoritative.

### Suggested implementation increments

1. Build a read-only parameter inspector against the existing schema, with
   manifest revision, destination, dependencies and truthful preview state.
   This needs no new template language or credential resolver.
2. Add a strictly bounded owner-side generation matrix and real behavior
   fixtures for the twelve built-ins. Report sampled versus exhaustive
   combinations; do not execute commands supplied by a template.
3. Add a small supported-language template pack only after source/license,
   prerequisites and generated-source tests are reviewed. Start with one
   standard-library HTTP handler or component behavior, not an entire SaaS
   application.
4. Specify multi-file publication and recovery in TinyTools before offering
   component-plus-test bundles. Keep every target revision and partial failure
   visible in the host receipt.
5. Extend the existing native/browser smoke suites for concrete remaining
   deployment seams. Keep artifact identity and actual runtime evidence
   separate from development-server and mocked-interface passes.

A full boilerplate repository can be offered as a separately maintained starter
only if the user chooses its framework and services. It is not a sensible hidden
dependency of a syntax tool. This investigation provides concrete candidate
contracts and validation work; it does not claim any of those future ports
have been implemented.
