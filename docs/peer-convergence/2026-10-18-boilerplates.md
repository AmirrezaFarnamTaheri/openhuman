# Boilerplate repository absorption audit

Snapshot: 2026-10-18 UTC. This is static source review of four named repositories plus a bounded repository-search sample. No external repository code was imported, executed, or installed.

## Recommendation

Prioritize three ideas: schema-driven chunk inspection and parameter forms; testing generated combinations as actual sources; and a desktop smoke test that verifies the renderer-to-host seam. Keep framework-specific deployment, authentication and state-management code in separate optional template packs after their prerequisites and ownership are specified. The current TinyTools engine stays a pure syntax/rendering library, with filesystem publication and policy in its native adapter.

Effort estimates below are engineering estimates for a focused implementation and tests by a developer familiar with OpenHuman. They exclude dependency upgrades and complete product rollout.

## Exact source snapshots

| Repository | Default-branch SHA | API last push | Top-level license |
| --- | --- | --- | --- |
| react-boilerplate/react-boilerplate | `d19099afeff64ecfb09133c06c1cb18c0d40887e` | 2023-03-23 | [MIT](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/LICENSE.md) |
| electron-react-boilerplate/electron-react-boilerplate | `e5b8242d8b21a4290c3a1f18ae3b64167e9de04a` | 2026-10-12 | [MIT](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/LICENSE) |
| ChristianLempa/boilerplates | `02d85a2d55c0e455d4af9b5af00984430e4fe404` | 2026-08-31 | [MIT](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/LICENSE) |
| aspnetcorehero/Boilerplate | `c5c9bcab020d2b18afcdd31e6345e465f0d5358a` | 2023-07-04 | [MIT](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/LICENSE) |

Metadata came from each repository's GitHub REST resource; exact heads came from `git/ref/heads/{default_branch}`. All four reported `archived=false`. Last-push dates describe repository activity, not a security or maintenance guarantee. Recursive trees were returned with `truncated=false`; this audit still reads selected files rather than claiming review of every file.

OpenHuman and TinyTools currently declare `GPL-3.0-only` in their workspace manifests. The four inspected MIT licenses permit copying, modification and redistribution subject to preserving the copyright and permission notices. This makes their own MIT-licensed source a plausible reuse candidate in the GPL project; this is a source-scope assessment, not clearance of every dependency, logo, bundled asset or downstream generated project. Preserve exact upstream attribution for any copied material. No attribution obligation is triggered by this audit because it imports no source.

## Concrete candidates

| Candidate | Evidence | Owning destination | Work and acceptance | Estimate / disposition |
| --- | --- | --- | --- | --- |
| Generate a component together with its behavioral test | [component generator](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/index.js); [test template](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/test.js.hbs) | TinyTools template pack, with OpenHuman preview | A TSX pattern plus required assertion slot, explicit destination paths and preview. Reject duplicate destinations and missing assertions. The upstream test contains a deliberately failing placeholder and skipped snapshot: write a real behavior test for OpenHuman instead of copying those placeholders. | 1–2 days; useful after multi-file application semantics are designed |
| Validate all supported generated variants | [generation/lint driver](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/scripts/generate-templates-for-linting.js); [dependency matrix](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/validation/dependency_matrix.py); [matrix tests](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/tests/test_dependency_matrix.py) | TinyTools fixture/CI tooling | Render every built-in default and bounded relevant parameter combinations, then parse and, where a package fixture exists, typecheck/test. Keep a fixed cap, identify sampled cases and refuse claims of exhaustive combination coverage. Do not execute arbitrary template-specified commands. | 1–2 days; strong near-term improvement |
| Parameter inspector with grouped, conditional fields | [typed variables](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/template/variable.py); [workflow documentation](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/README.md) | OpenHuman UI; typed metadata extensions owned by TinyTools | Display declared type, default, description and required status before applying. Future enum/range/group/dependency metadata needs a versioned schema and deterministic validation. Start with existing six parameter kinds. Preview must display the exact revision and destination. | 2–4 days; high product value |
| Secret-reference parameter type | [secret metadata](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/cli/core/template/variable.py) | OpenHuman credential boundary, then owner schema | Carry a secret reference rather than embedding plaintext defaults or logging values. Upstream rejects defaults for autogenerated secrets. Define redacted inspect/preview behavior and host resolution before adding this kind. | 2–4 days; requires a separate accepted contract |
| Desktop startup and bridge smoke test | [desktop smoke runner](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.erb/scripts/smoke-test.cjs); [OS matrix](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.github/workflows/test.yml) | OpenHuman app CI | Assert the main route, loaded assets, no runtime exceptions, and a bounded renderer-to-host round trip. Translate the checks to Tauri's existing runtime/test seam. Keep deadlines and process cleanup. The upstream uses Electron CDP and IPC; its exact runner is platform-specific. | 2–3 days; strongest build-system idea |
| Typed subscription API with explicit cleanup | [preload bridge](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/src/main/preload.ts) | OpenHuman frontend host-event adapters | Return a disposer from subscriptions, constrain event names, test unmount cleanup and duplicate subscriptions. The source is a useful API pattern; Tauri must continue using its own command/event authorization. | 0.5–1.5 days; use when an actual cleanup gap is found |
| Release-stage verification across operating systems | [workflow](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.github/workflows/test.yml); [build configuration](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/package.json) | OpenHuman release CI | Separate source checks from package construction and runtime smoke. Attach exact artifact and source identities to receipts. Map macOS signing and Linux display requirements to existing Tauri workflow rather than adding a second desktop runtime. | 2–4 days; compare existing release gates first |
| A permission/audit inspection panel | [permission table](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Areas/Admin/Views/Permission/Index.cshtml); [permission names](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Application/Constants/Permissions.cs) | OpenHuman settings / run receipts | Group capabilities and show effective decisions and provenance. Preserve OpenHuman's actual dynamic tool policy, including dry-run classification. Do not translate CRUD role claims into tool authorization by assumption. | 2–4 days; presentation idea only |
| Explicit application/domain/infrastructure boundaries in generated services | [architecture](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/README.md); [authorization implementation](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Permission/PermissionAuthorizationHandler.cs) | Optional service template pack | Produce small tested Rust/TypeScript equivalents only when requested. The existing C# application and ASP.NET authorization stack do not belong inside TinyTools or the Rust host. C# is outside the current grammar set. | 3–5 days per reviewed service pattern; later |

## Maturity and fit of each repository

### React Boilerplate

The pinned package is version 4.0.0 and declares React 16.8.6, Redux 4.0.1, Redux-Saga 1.0.2 and a Webpack-based build. Its generator can choose memoization, translated messages and asynchronous loading, and generates paired test files. These are concrete template-design ideas. Its dependencies and test scaffolding require modernization before execution in the current host; the audit did not run them. The useful unit of reuse is a reviewed generated pattern and its assertions. [Package evidence](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/package.json); [generator evidence](https://github.com/react-boilerplate/react-boilerplate/blob/d19099afeff64ecfb09133c06c1cb18c0d40887e/internals/generators/component/index.js).

### Electron React Boilerplate

The current snapshot uses Electron/Vite, TypeScript, React 19.3 and a three-OS CI matrix. The smoke runner verifies a rendered page, image, links, absence of a Vite error overlay, an IPC round trip and absence of renderer exceptions. This is a concrete example of testing the shipped application seam. OpenHuman uses Tauri 2, so the most valuable transfer is the verification structure and typed cleanup API. Introducing Electron, its updater or its packaging tree would create a new runtime to own. [Package](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/package.json); [smoke checks](https://github.com/electron-react-boilerplate/electron-react-boilerplate/blob/e5b8242d8b21a4290c3a1f18ae3b64167e9de04a/.erb/scripts/smoke-test.cjs).

### Christian Lempa Boilerplates

This is the closest match to a discover/inspect/configure/render product flow. Its variable model includes descriptions, types, defaults, options, ranges, dependency conditions and special secret handling. Its matrix builder caps combinations and has dedicated tests.

There is a material migration boundary at the inspected SHA: the README says version 0.2.0 uses `template.json` and `files/`, and legacy YAML manifests and .j2 files are unsupported. The same tree still contains old `library/**/template.yaml` entries, including a PostgreSQL entry marked draft. The README now points to a separate template-library repository. Therefore bulk ingestion of this repository's legacy library would be unsound. A future importer must pin the supported format, inspect the separate library, and validate each selected template. [Format contract](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/README.md); [legacy draft example](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/library/compose/postgres/template.yaml); [new-format fixture](https://github.com/ChristianLempa/boilerplates/blob/02d85a2d55c0e455d4af9b5af00984430e4fe404/tests/test_dependency_matrix.py).

### ASP.NET Core Hero

The README targets ASP.NET Core 5 / Entity Framework Core 5 and advertises MVC/WebAPI, role-claim permissions, audit activity, localization, RTL and dark-mode UI. The inspected permission handler checks claims with a fixed local issuer; this is a web-application authorization design, not a substitute for dynamic tool admission. Its permission table and boundary organization can inform OpenHuman UI and future service patterns. No framework, authentication or database implementation should move into the syntax engine. [README](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/README.md); [handler](https://github.com/aspnetcorehero/Boilerplate/blob/c5c9bcab020d2b18afcdd31e6345e465f0d5358a/AspNetCoreHero.Boilerplate.Web/Permission/PermissionAuthorizationHandler.cs).

## Requested search snapshot

Requested URL: https://github.com/search?q=boilerplate&type=repositories&p=2

The request specifies repository search, query `boilerplate`, page 2, and no explicit sort or filters. It is a discovery view with changing rank/order, not an exhaustive inventory. The GitHub connector rejected the HTML search URL and web retrieval could not open it. To make the scope reproducible, I used the connector repository-search API with `query=boilerplate,page=2,per_page=10`, leaving sort unspecified. This is an API discovery sample, not a claim to have reproduced the browser page's exact membership.

The API returned these ten repository identities:

- [aspnetboilerplate/aspnetboilerplate](https://github.com/aspnetboilerplate/aspnetboilerplate)
- [ixartz/Next-js-Boilerplate](https://github.com/ixartz/Next-js-Boilerplate)
- [hagopj13/node-express-boilerplate](https://github.com/hagopj13/node-express-boilerplate)
- [h5bp/server-configs-nginx](https://github.com/h5bp/server-configs-nginx)
- [4GeeksAcademy/html-hello](https://github.com/4GeeksAcademy/html-hello)
- [freeCodeCamp/boilerplate-express](https://github.com/freeCodeCamp/boilerplate-express)
- [gofiber/boilerplate](https://github.com/gofiber/boilerplate)
- [DevinVinson/WordPress-Plugin-Boilerplate](https://github.com/DevinVinson/WordPress-Plugin-Boilerplate)
- [ixartz/SaaS-Boilerplate](https://github.com/ixartz/SaaS-Boilerplate)
- [genlayerlabs/genlayer-project-boilerplate](https://github.com/genlayerlabs/genlayer-project-boilerplate)

Those ten received identity-level discovery only. Their licenses, source heads and implementations have not been audited here. The four explicitly requested repositories above received the detailed source review. If expanding the search, first inspect h5bp's Nginx configuration tests, GoFiber's service entry points, and the Node/Express validation/test structure because they may map to concrete configuration or HTTP-handler template packs. That is a prioritization hypothesis, not an endorsement of unreviewed source.

## Implementation boundary

The current PR work should finish its owner tests, strict lints, coverage and source publication first. Then propose a separate, versioned expansion for multi-file bundles, typed form metadata and bounded variant testing. Every future imported pattern needs an upstream source SHA/license record, a supported language, typed inputs, declared destination, meaningful fixtures, and explicit host publication policy. Existing source revision checks, one-pass expansion, preview classification and filesystem gates remain authoritative.

