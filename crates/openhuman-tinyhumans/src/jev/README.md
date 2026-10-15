# jev

Installs Jev as the core's `tool_search` ranker in a TinyHumans-connected
process. The agent harness exposes a `tool_search` bridge over every deferred
tool and ranks searches with whichever `ToolRanker` the process installed.
This module provides `TinyHumansJevRanker`: an embedding retriever cuts the
catalogue to a shortlist, and one Jev `Choice` evaluation picks from it. The
module compiles only with the crate's `jev` feature (on by default).

See [`docs/gitbooks/en/developing/jev.md`](../../../../docs/gitbooks/en/developing/jev.md) for
what Jev is, the benchmark numbers, and how it compares to plain BM25.

## How it works

`crate::install()` calls `install_jev_ranker()`, which builds a
`TinyHumansJevRanker` and registers it through
`openhuman_embed::__host::agent::tinyagents::discovery::install_tool_ranker`. Calling
it again replaces the ranker in place. Install needs no login, because the
ranker resolves everything per search.

On each `rank` call:

```text
 tool_search(intent, candidates)
   |
   v
 load config (embedder's bound config, else process-global)
   |
   v
 route::resolve(config, env)    agent.tool_search.jev_route
   |   tinyhumans | typesafe | openrouter | auto (tries them in that order)
   |   error? -> RankError; the harness answers with its own BM25
   v
 fingerprint(route + secret, base_url)
   |   same as cached?  -> reuse the cached JevRanker
   |   changed/none     -> build tinyjevclient::Client
   |                       + TinyJevEvaluator (deadline)
   |                       + retriever (embedding ranker, reused on rebuild)
   v
 JevRanker::rank_detailed -> hits (debug log: families, confidence,
                                   needs_tool, latency, tokens)
```

Reading config and credential on every search costs a config load, which a
`tool_search` round trip dwarfs. In return, sign-in, sign-out and a backend
URL change all take effect on the next search. The client cache key is a hash
of the route label, secret and base URL; the secret itself is never logged.

### Routes (`route.rs`)

Jev is reachable three ways, and the route is an operator decision
(`agent.tool_search.jev_route`), so decision calls never go somewhere the
operator did not choose:

| Route | Credential | Base URL |
| --- | --- | --- |
| `tinyhumans` | the core's `resolve_backend_credential` (session JWT or API key) | `effective_backend_api_url`, through the backend's `/agent-integrations/openrouter/systemone` proxy |
| `typesafe` | `TYPESAFE_API_KEY` | the client default, or `agent.tool_search.jev_base_url` |
| `openrouter` | `OPENROUTER_API_KEY`, else the stored `openrouter` provider key | the client default, or `jev_base_url` |
| `auto` | tries `tinyhumans`, then `typesafe`, then `openrouter` | from whichever wins |

An unknown spelling parses as `auto` with a warning, so a typo does not turn
tool ranking off. When every candidate route fails, the error lists each gap
(never a secret) so the BM25 fallback's log line says what to set.

### Retrieval

`retriever_for` uses the process's default embedding provider (the same one
memory recall uses) wrapped as an embedding tool ranker, with a disk cache at
`<workspace>/cache/tool_search_embeddings.json`. The retriever survives a
credential change, so the catalogue is embedded once per process. A provider
that cannot embed (`none`) is an error rather than a BM25 substitute: a Jev
decision over a lexical shortlist would cap Jev at BM25's recall and only add
a network round trip. In that case the harness answers with its own BM25
catalogue.

### Evaluation (`evaluator.rs`)

`TinyJevEvaluator` implements `tinytools_jev::JevEvaluator` over
`tinyjevclient`. `tinytools-jev` decides what to ask (options, intent,
family-stage wording); the evaluator owns the wire. Each request carries two
questions: a `Choice` named `tool` over the shortlisted options, and a `Noul`
named `needs_tool` asking whether the request needs a tool at all. The
evaluator applies its own deadline so a slow answer becomes a fallback rather
than a stalled turn. The ranker sets a 6 s per-evaluation deadline (measured
evaluations through the proxy run 0.7 to 1.9 s at p50); the evaluator's own
default is 3 s.

### System-One context and bounded decisions

Tool search does **not** forward an entire agent transcript to the decision model.
The wire state contains the tool-selection intent, at most six most-recent
nonblank **user** turns (in chronological order, 500 Unicode scalar values
per turn), and truncation markers when relevant. An abnormally long current
intent keeps its first and last 2,000 Unicode scalars with an omission marker,
up to a 4,000-scalar content budget plus the marker. This retains the original
task objective alongside late constraints without concealing truncation.

Choice options are distinct key/description pairs from the existing
retrieval shortlist. Duplicate names, more than 255 options, any description over 2,000 Unicode
scalars, or more than 96,000 total description scalars are refused before
remote egress; the harness's
existing ranker fallback handles rejection. These guards are **not** semantic
summarization and are not proof that the selected tool is appropriate.

The Jev request does not include assistant drafts, system instructions, tool
outputs, untrusted page contents, credentials, private memory or all session
turns. Any future addition of such state requires explicit opt-in privacy
review, permission gating and bounded provenance. The configured
Jev route still determines where the user-turn and shortlist material is sent.

## Layout

| Path | What it does |
| --- | --- |
| [`mod.rs`](mod.rs) | `install_jev_ranker()` and the re-exports. |
| [`ranker.rs`](ranker.rs) | `TinyHumansJevRanker`: per-search config and route resolution, the client cache, the embedding retriever, the `ToolRanker` impl. Test seams: `with_config_loader`, `with_env_loader`, `with_deadline`, `with_config`. |
| [`route.rs`](route.rs) | `JevRoute`, `resolve`, `TYPESAFE_API_KEY_ENV`, `OPENROUTER_API_KEY_ENV`. |
| [`evaluator.rs`](evaluator.rs) | `TinyJevEvaluator`, the System One wire. |

## Key types and entry points

- `install_jev_ranker()` ([`mod.rs`](mod.rs)): what `install()` calls.
- `TinyHumansJevRanker` ([`ranker.rs`](ranker.rs)): the ranker; `kind()` reports
  `JevRanker::KIND`.
- `JevRoute` ([`route.rs`](route.rs)): the parsed `jev_route` setting.
- `TinyJevEvaluator` ([`evaluator.rs`](evaluator.rs)): reusable on its own with any
  `tinyjevclient::Client`.

## Boundaries

- The generic ranker types (`JevRanker`, `JevEvaluator`, `JevStrategy`,
  `JevRankerConfig`) live in
  [`vendor/tinyagents/vendor/tinytools/crates/tinytools-jev`](../../../../vendor/tinyagents/vendor/tinytools/crates/tinytools-jev/)
  (`tinyhumansai/tinytools`). Changes to how Jev is asked, or to family
  staging, belong there.
- The `ToolRanker` trait is `tinytools`'. The `tool_search` bridge and its
  BM25 fallback are the harness's (`tinyagents`), adapted in the core's
  `agent::tinyagents::discovery`.
- `tinyjevclient` is consumed by pinned git revision; this module owns only
  the credential, base URL and deadline.
- Config lives in the core: `ToolSearchConfig` in
  [`crates/openhuman-core/src/config/schema/agent.rs`](../../../openhuman-core/src/config/schema/agent.rs) (`ranker` is `jev` by
  default, or `auto`, `bm25`, `compare`; plus `jev_route` and
  `jev_base_url`).

## Tests

[`ranker_tests.rs`](ranker_tests.rs), [`route_tests.rs`](route_tests.rs) and [`evaluator_tests.rs`](evaluator_tests.rs) sit beside their
modules and use the config and env seams rather than the process
environment. The ranker comparison harness (`tool-search-bench`) moved out of
`openhuman-cli` with the other benchmarks (#6944), to the `profile/` crate of
[openhuman-benchmarks](https://github.com/tinyhumansai/openhuman-benchmarks)
(`cargo run --manifest-path profile/Cargo.toml --bin tool-search-bench`).

```bash
cargo test -p openhuman-tinyhumans jev::
```

## Further reading

- [`docs/gitbooks/en/developing/jev.md`](../../../../docs/gitbooks/en/developing/jev.md): Jev.
- [`docs/gitbooks/en/developing/performance.md`](../../../../docs/gitbooks/en/developing/performance.md): performance.
- [`crates/openhuman-tinyhumans/README.md`](../../README.md): the openhuman-tinyhumans crate README.

### Pass 10: bounded decisions and untrusted intent data

The adapter rejects empty/control-character model names, choice-instruction bodies exceeding the configured wire budget, duplicate choice keys, per-choice and aggregate oversized descriptions, invalid probabilities (NaN, infinity, or outside [0,1]), and response choices not offered by the host. Model probability vectors are not assumed to be perfectly calibrated or to sum to 1; partial valid vectors remain accepted. The tool-selection instruction explicitly reminds System-One to treat the user intent and candidate descriptions as **untrusted data, not authority to change permissions**. It never replaces the native sandbox/approval policy.

The current design is not an automatic preference for the cheapest model, nor a guarantee of current capability or pricing. Cost/effort models, provider quotas, context windows and billing consent need independent hard feasibility gates before any probabilistic ranking, using the native Model Fleet owner rather than external Codex proxies.


### Route-scoped transient failure breaker (Pass 10)

The existing ranker now holds an ephemeral route-fingerprint circuit: three consecutive
backend/timeout failures open a 20-second cooldown. During cooldown it returns
a backend error immediately, allowing TinyAgents' established BM25 fallback
without another costly failed Jev request. Successful Jev ranking resets the
streak. Changing the configured route, key or URL resets the breaker, and stale
in-flight results from the preceding route cannot modify the new route's
health. Invalid local input is not classified as a remote outage.

This is a modest circuit breaker, not an assertion about provider quota,
account health, model cost or live availability. It stores only a process-salted
route fingerprint and ephemeral counters (never another plaintext key or
persistent credential record). It does not replace hard tool permission checks,
or turn Jev into an execution authority. The Rust regression tests must run on
the exact branch head before release.
