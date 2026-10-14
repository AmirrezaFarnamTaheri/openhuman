# Native 2API protocol compatibility (Qwen/QNML)

> **Status:** Opt-in QNML compatibility implemented on the draft convergence branch.
> This is a protocol adapter, **not** a built-in anonymous or consumer-session API.
> Live upstream smoke tests and the latest full CI run remain release gates.

## Use from OpenHuman

1. In **Settings → AI → Providers**, add or edit the endpoint of an explicitly
   authorized OpenAI-compatible 2API gateway.
2. Keep **Tool-call format: Native API (recommended)** if it emits structured
   OpenAI `tool_calls`, Anthropic Messages tool blocks, or native Responses events.
3. Choose **QNML tags (Qwen / 2API compatible)** only when that provider's text
   output contains Qwen-style tagged tool invocations. Save and test the provider.
4. Select the provider/model in OpenHuman's existing Model Fleet. Keys remain in
   the encrypted/scoped credential store and existing route overrides still apply.

Example of the bounded accepted 2API QNML envelope:

```text
<|QNML|tool_calls>
  <|QNML|invoke name="read_file">
    <|QNML|parameter name="path"><![CDATA[README.md]]></|QNML|parameter>
  </|QNML|invoke>
</|QNML|tool_calls>
```

The native adapter converts a **complete** envelope into typed
`tinyinference_llm::tool::ToolCall` objects only after checking the name
and every argument against the tool declarations in the original request.
Model text is never interpreted as a tool call when the mode is unset.

## Ownership and security boundaries

- The existing Rust factory, `ProviderRegistry`, key pool, TLS/egress policy,
  cancellation and Model Fleet are the **single owners** of transport and auth.
- The QNML adapter is a small `ChatModel<()>` wrapper around an already-built
  native model, selected per provider via `CloudProviderCreds.tool_call_codec`.
- ToolChoice::None disables conversion; ToolChoice::Tool(name) restricts the
  decoder to the selected tool. Tool schemas validate parsed arguments.
- Ordinary prose/code examples quoting QNML cannot invoke tools: the complete
  top-level envelope must be present. Unknown names, broken tags, duplicate
  arguments, wrong argument types and unclosed CDATA fail closed.
- At most **256 KiB** total tool-output, **16** calls, **48** arguments per call,
  and **64 KiB** per parameter are inspected. No permissive lookalike/
  Unicode/homoglyph repair is performed before assigning executable calls.
- Tool-enabled streams are buffered until an authoritative terminal response;
  no partial tool markup escapes to the chat UI. HTTP provider failures,
  deferred handles, and cancellation stay inside the existing stream lifecycle.
  Normal text requests without tool declarations retain incremental streaming.
- The adapter does not execute tools. All tool actions still flow through
  OpenHuman's normal agent runtime approvals, permissions and audit surfaces.
- Disabling QNML changes only the selected provider's metadata. All previously
  configured routes, image model endpoints, model registry and credentials
  remain unchanged. Legacy provider settings default to native formatting.

## Source inspiration and deliberate exclusions

| Peer | Used behavior / evidence | What was deliberately *not* imported |
| --- | --- | --- |
| `qwen2API/backend/toolcall/formats_qnml.go` | QNML invoke/parameter envelope, CDATA values and multi-call examples | Heuristic lookalike markup repair, automatic undeclared-tool insertion |
| `web2api/core/api/tagged_stream_parser.py` | Explicit stream phases and failure on incomplete tool blocks | A parallel Python websocket/HTTP server and broad content scraping |
| `grok2api/backend/internal/transport/http/inference/responses_compat.go` | Streaming compatibility and never replaying already-visible events | A second gateway/egress manager, unauthenticated public relay |
| `gemini-web2api-go/internal/app/cookie_pool.go` | Account-affinity analysis | Browser-cookie extraction, device identity impersonation |
| `deepseek-free-api/deepseek/pool.py` | Key cooldown and account health review | Separate long-lived plaintext credential store |

Rust is the appropriate owner for this feature because every one-shot, streamed
and agent invocation already crosses the same Rust `ChatModel` interface.
Introducing a new Go sidecar merely for tagged output would duplicate process,
auth, updates and lifecycle handling. Go remains a sensible choice for a future
**explicitly consented, isolated site-specific connector** when native HTTP
transport is insufficient, but it is not a prerequisite for this codec.

## Tests and remaining gates

Rust regression cases cover strict QNML parsing, CDATA preservation, multiple
validated calls, unknown/malformed/truncated markup, length limits, a native
tool-call passthrough, tool_choice restrictions, buffered tool-enabled streams
and unmodified ordinary-text streaming. Frontend tests check the settings
serialization and provider-editor save/rollback path. Separate fleet regression
cases cover 429 retry/lease behavior and the no-replay rule after streamed text.

The full pinned Rust workspace (including tinyinference submodules), frontend
typecheck/Vitest, Tauri packaging, changed-line coverage and disposable-key
gateway smoke tests must pass before claiming production-ready parity.

### Outstanding convergence capabilities

- Tagged-JSON tools (`web2api`) and tolerant read-only diagnostic previews
  without ever promoting untrusted malformed text to executable calls.
- Explicit model-level protocol preference (Chat Completions vs Responses vs
  Anthropic Messages/Gemini-native), with provenance and expiration.
- Typed thinking/tool/media/usage event differential fixtures, SSE chunk
  fuzzing and cross-provider cancellation/partial-output semantics.
- Opt-in authorized web-account adapter connectors with isolated tokens,
  artifact IDs and revocable session/consent; **no silent web-cookie reuse**.
