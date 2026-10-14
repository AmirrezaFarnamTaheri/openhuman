# Peer-fleet contracts for OpenHuman

This is a self-contained, **credential-free** JavaScript ESM integration layer prepared from the local four-peer audit. It is not a replacement for OpenHuman's Rust provider runtime, Tauri process host, secret vault or the optional Prodex gateway. It has no external npm dependencies. Node.js 22 or newer is required for local tests; the pure modules can be adapted into a frontend/runtime boundary after target-source inspection.

## Module ownership

- `catalog.mjs`: provider-qualified model declarations vs authenticated live availability, strict model identity, alias cycles, verified-only price/capability fields.
- `quota.mjs`: normalized provider/account quota windows, stale observations, model-independent freshness, conservative usage runway estimates.
- `routing.mjs`: an advisory account ranking algorithm, bounded direct-key leases, one-account cooldown and postcommit-retry avoidance; a delegated `prodex` route selects no account.
- `presets.mjs`: credential-free model/endpoint presets and multiple-model groups; revisioned atomic batch edits, remote-sync conflict detection, tombstones and endpoint validation.
- `refresh.mjs`: coalesced single-flight refresh and bounded retry scheduling metadata, not live OAuth handling.
- `widgets.mjs`: normalized quota-table columns, context-window gauges, reset-aware usage buckets and aggregate dashboard data.
- `convergence.mjs`: second-order composition of the above into a provider-scoped fleet view without sending any account tokens to the UI.
- `peer-declared-models.json`: 48 declarations referring to 35 peer-supplied model IDs; they are **unverified hints**, not a catalog of available/paid/free models. Source paths accompany each declaration.

## Examples

```js
import {buildCatalog, buildDashboard, recommendModelRoute} from './index.mjs';
const live = []; // Supply only after an authenticated provider/gateway discovery call.
const catalog = buildCatalog([{provider:'antigravity',id:'gemini-3-flash',source:'peer'}],live);
console.log(catalog[0].availability); // unverified
const now = Math.floor(Date.now()/1000);
const accounts = [{id:'opaque-id',provider:'antigravity',models:['gemini-3-flash'],health:'healthy',quota:{fetchedAt:now,windows:[{durationSeconds:18000,usedPercent:30}]}}];
console.log(buildDashboard(accounts,{now}).accounts[0].remainingPercent); // 70
console.log(recommendModelRoute({accounts,provider:'antigravity',model:'gemini-3-flash',now}));
```

No secrets should be included in `accounts`, `presets` or provider-model declarations. The view model allowlists account fields; the routing controller similarly discards unknown account fields. The host retains the actual credential vault and transport.

## Tests

```bash
node --test packages/peer-fleet/test/*.test.mjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/ci/tests -p 'test_*.py' -v
```

The workflows use offline, read-only checks. They cannot demonstrate a running OpenHuman desktop UI, real provider entitlement, production webhook, CodeRabbit thread, or successful GitHub CI run.


## Antigravity Manager and Cockpit Tools adapters

- \`observations.mjs\`: normalizes already-fetched quota DTOs into account-scoped, freshness-aware independent model and grouped windows. It never guesses account tiers from model names or treats unknown quota as exhausted.
- \`telemetry.mjs\`: deduplicates upstream attempt diagnostics, distinguishes incomplete streams from completed output, computes qualified approximate throughput and redacts request data.

Neither adapter fetches OAuth information, mutates IDE account files, runs a relay, or overrides OpenHuman/Prodex key selection. Source projects disclose CC BY-NC-SA 4.0; no upstream implementation source was copied into this independent contract layer.
