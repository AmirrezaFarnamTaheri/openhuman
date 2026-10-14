# API peer convergence — second-order decisions

> **Date:** 2026-10-14. **Target:** OpenHuman (the Rust core remains the policy/runtime owner).
> **Scope:** Sixteen user-provided archives. This is an evidence-driven integration ledger,
> not a statement that every upstream feature is semantically equivalent or fully ported.
> See the conversation's SHA-256 file census for 1,370 indexed archive files.

## Completed on `feat/peer-provider-convergence` (draft PR #8)

1. Six built-in OpenAI-compatible service presets are synchronized across
   `app/src/components/settings/panels/builtinCloudProviders.ts` and
   `crates/openhuman-core/src/config/schema/cloud_providers.rs`:
   Aion Labs, SambaNova, OVHcloud AI Endpoints, Nebius Token Factory, LLM7.io and
   Synthetic (the documented successor of glhf.chat). These are *endpoint hints*,
   **not** guarantees of free access, accepted credentials, model entitlement, quotas,
   streaming support or stable pricing.
2. `scripts/ci/check-cloud-provider-catalog-parity.mjs` and the static CI lane
   reject drift across the **full pinned vendor + Rust extras + React UI**
   catalog (26 vendor, 12 Rust extras, 38 frontend presets at this pin),
   including identities, endpoints, authentication methods, missing/ghost UI
   entries, and conflicting duplicate slugs. The guard audits all shipped
   preset URLs for remote HTTP, embedded credentials, query/fragment secret
   channels and unsupported schemes, while allowing intentional loopback HTTP.
   A fake-repository CLI test verifies source-root resolution independent of cwd.
3. The Rust Responses endpoint capability gate now includes Nebius, OpenRouter
   and xAI alongside OpenAI and Prodex. The pinned vendor knows OpenRouter and
   xAI; official API docs confirm these endpoint families. This is **not**
   evidence that every concrete model supports every Responses feature.
4. Credential Fleet selection and lease reservation are atomic for requests,
   including streams, so least-busy uses in-flight concurrency rather than
   simultaneous readers observing an idle key. Legacy non-reserving selection
   remains available to read-only catalog probes.
5. Concurrent 429s preserve the longest active cooldown. A late success from
   another in-flight request does not cancel a 429. Session affinity eviction
   selects the earliest expiry. Per-invocation attempts are bounded to avoid
   multiplying provider calls by every configured API key.
6. New regression tests exercise the catalog, native model aliases, key-pool
   race conditions, streaming/unary bounded retry and credential-lease lifecycle.
7. Cloud-provider 403 classification is now conservative: exact credential
   errors evict one key, exact quota failures cool one key down, and unrelated
   forbidden errors (including `invalid_model` and `invalid_request_error`)
   do not fan out one failing invocation across the fleet. Unary and streaming
   regression cases cover the selection boundary.
8. Provider-specific image endpoint overrides require HTTPS for remote hosts
   because inference credentials can be reused on image routes. Plain HTTP is
   retained only for explicit loopback development gateways
   (`localhost`, `127.0.0.0/8`, `::1`).
9. An opt-in Rust QNML `ChatModel` codec now translates complete, bounded
   Qwen/2API text tool invocations into schema-validated native `ToolCall`
   objects, with tool-choice restrictions and buffered tool-enabled streaming.
   Settings remain in the existing React provider editor and core config; no
   sidecar credential store is introduced. See
   [Native 2API compatibility](NATIVE_2API_COMPATIBILITY.md) for usage, test
   contracts, limitations and later protocol adapters.


Do **not** confuse unit/source assertions with live provider validation. As of this
continuation, Rust/Tauri full-suite and approved-key provider smoke results for the
latest branch head have not been independently confirmed.
Required Rust/React/Tauri CI and disposable-key integration smoke tests remain
release gates. Full other-peer feature convergence is not represented as done.

## Peer → OpenHuman disposition

| Peer archive | Concrete evidence inspected | Useful capability | Adoption boundary |
| --- | --- | --- | --- |
| GhostBrain | `GhostBrain.py` | request/stream lifecycle and response termination | typed native inference adapter; no main-process browser credential ownership |
| WebAI2API | `src/backend/pool/Worker.js`, `src/backend/adapter/lmarena_text.js` | worker limits and isolation | existing worker/scheduler, no copied fingerprint-evasion subsystem |
| awesome-freellm-apis | Markdown catalog, not executable adapter code | discovery candidates | reviewed suggestions only; never imply a static free entitlement |
| chat2api | `chatgpt/chatFormat.py`, `gateway/backend.py` | Chat Completions response mapping | isolated codec fixtures; no imported consumer-session secrets |
| deepseek-free-api | `deepseek/pool.py`, `server/anthropic.py` | cooldown/account-pool policies | central Rust key-pool and explicit provider authorization |
| gemini-free-api | `internal/gbridge/agent.go`, `internal/gbridge/anthropic.go` | Gemini/Anthropic message semantics and tool results | narrow native codec, not a second authorization control plane |
| gemini-web-to-api | `internal/modules/providers/gemini_service.go`, `gemini_video.go` | media, deep research, multi-step operations | capability-gated media adapters + native artifact storage |
| gemini-web2api | `gemini_web2api/server.py`, `gemini_web2api/multimodal.py` | model discovery, media handling and Responses cases | fixture-backed protocol tests only until upstream transport is authorized |
| gemini-web2api-go | `internal/app/cookie_pool.go`, `internal/app/messages.go` | affinity and session scheduling | use native scoped credential/session store; no silently imported browser cookies |
| grok2api | `backend/internal/application/gateway/service_test.go`, `backend/internal/infra/egress/` | gateway governance, egress classification and rich media tests | keep existing OpenHuman policy/secret and API ownership |
| opencode-edge | `worker.js` | OpenAI/Anthropic stream interop and alias mapping | reject public unauthenticated relay defaults and wildcard credentialed CORS |
| public-api-lists | Archive lacks executable catalog entries | potential catalog validation conventions | no API data to import |
| public-apis | `scripts/validate/format.py`, `scripts/validate/links.py` | catalog format/link linting | offline provider-list quality gates only; archive lacks provider payload |
| qwen2API | `backend/toolcall/formats_qnml.go`, `backend/context_pipeline.go` | QNML parsing, tool-call ID stability and context limits | tested provider-specific parser behind native tool-event contracts |
| toapi | `toapi/api.py`, `toapi/item.py` | declarative site extraction | opt-in scrape tool, enforce SSRF/robots/size limits; not LLM routing |
| web2api | `core/api/tagged_stream_parser.py`, `core/protocol/anthropic.py` | incremental tool/SSE parsing and tagged blocks | typed streaming codec with bounded buffering and cancellation |

## Second-order architecture after provider consolidation

```text
Desktop / CLI / TUI
      | (user-selected model, workload, permissions)
      v
OpenHuman Rust policy: provider route + scope + credentials + approval gates
      | (one request attempt budget and traced correlation/run identity)
      v
Model capability resolver [provider, concrete model, endpoint, protocol]
      | (live facts first; unverified metadata is "unknown")
      v
Provider codec [Chat Completions | Responses | Messages | Gemini]
      | (typed text/thought/tool/media/usage/terminal events)
      v
Network adapter + pool lease + bounded retries
      | (no replay after visible content, tool action or deferred handle)
      v
Native artifact/session store, cost telemetry, UI result
```

All layers are **contracts inside the existing core**, not standalone servers.
No adapter gets to bypass approval, read arbitrary credentials, impersonate a
different account or rewrite routing/billing state.

### Highest-priority remaining work

**P0 — protocol-/model-level routing.** OpenCode Zen and other mixed protocol
gateways may require different transport for different models. Introduce a
versioned capability record keyed by `(provider, model, endpoint)` recording
Chat Completions, Responses, Anthropic Messages, Gemini-native, input modalities,
tool calls, reasoning, explicit provenance and expiry. An unknown model must not
be advertised as fully supported. Only route to a different transport when it
is explicitly discovered or manually configured. Test tool-call, streaming,
refusal, cancellation, usage and error parity on each surface. Do not infer
model capability from slug alone.

**P0 — failover and idempotency.** Keep the one-attempt-budget and atomic lease.
Enforce no retries once tokens, tool side effects, deferred job handles, or
account-bound upstream file IDs have escaped. Include race tests for
delete/re-add key, stale 429, key rotation, hot config reload, stream cancellation
and Retry-After bounds.

**P1 — codec adapter extraction.** Compare QNML/tagged tool parsing from
`qwen2API` and `web2api` against native Rust inference's typed tool events.
Port only an opt-in parser module if tests demonstrate missing native behavior.
Cap buffer sizes, reject malformed tool commands, keep reasoning blocks distinct
from tool arguments, and fuzz untrusted tags/corrupt SSE chunks.

**P1 — provider account connectors.** Give each authorized connector a narrow
secrets namespace and user-consent flow. Preserve sticky sessions for uploads,
long-running research/video and token/cookie-bound context; prohibit automatic
transfer of web-consumer auth or browser fingerprint spoofing. Surfaces must
distinguish configured, connected, quota limited, unavailable and unknown.

**P2 — multimodal/artifacts and catalog UX.** Add provider media routes through
registered capabilities and private artifact storage. Offer source provenance,
last verified time, checkable live health, per-model cost and explicit
free-status uncertainty. Keep dynamic catalogs separate from default presets.

## Acceptance and migration

- The checked-in branch and `main` remain separate; PR #8 stays **draft**.
- CI must pass on the latest *exact* head, not a superseded or cancelled run:
  Rust fmt/clippy/coverage, frontend typecheck/lint/Vitest, static Node checks,
  Tauri build and changed-line-coverage gate.
- Integration tests must use disposable keys and approved test endpoints, with
  negative coverage for credential disclosure, SSRF, rate limits, partial SSE,
  retries after output, and model capability mismatch.
- Do not relabel archive material as MIT solely from an assumption; maintain
  provenance for any adapted code and resolve licensing before release.
- No claim of exhaustive semantic review, all 16 repos merged, complete
  feature parity or production readiness until independently verified.
