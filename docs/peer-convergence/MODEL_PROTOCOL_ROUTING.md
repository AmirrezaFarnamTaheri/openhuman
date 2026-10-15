# Explicit per-model OpenAI transport routing

This document consolidates the model-scoped Responses transport introduced
in PR #9 with the broader model-fleet implementation in PR #8. It describes
**user-configured** capabilities, not automatic protocol discovery.

## Persistent configuration

`cloud_providers[].model_transports` maps an *exact* upstream model ID to
one of three transport modes:

```json
{
  "slug": "mixed-protocol-gateway",
  "endpoint": "https://gateway.example/v1",
  "auth_style": "bearer",
  "model_transports": {
    "vendor/responses-only": "responses",
    "vendor/legacy-chat": "chat_completions",
    "vendor/default-route": "auto"
  }
}
```

| Mode | Routing behavior |
| --- | --- |
| `auto` | Retain the pinned provider registry's native behavior, including pre-existing OAuth/Anthropic routing. This is the default for every unlisted model. |
| `chat_completions` | Explicitly select the existing OpenAI-compatible Chat Completions adapter; do not silently force a Responses route for the named model. |
| `responses` | Explicitly select the pinned native OpenAI Responses adapter for that model, posting to the configured endpoint's `/responses` path. |

No model prefix, slug, family, apparent provider capability or other model's
success authorizes a protocol for a sibling. Mapping keys are exact upstream
wire IDs. Unknown models and legacy saved configurations retain `auto`.
An empty map is omitted on serialization.

Native RPC normalization rejects empty or padded model IDs, control
characters, IDs over 200 UTF-8 bytes, and more than 512 entries. The advanced
editor uses the same bounds and rejects duplicate IDs.

## Authentication and security

Explicit `chat_completions` / `responses` routes require an
OpenAI-compatible `bearer` or `none` authentication style. Native Anthropic
Messages and managed JWT providers preserve `auto` only. Codex OAuth
requires its existing native Responses/account route and must not be forced
into Chat Completions.

The model-fleet factory retains OpenHuman as the single owner of provider
credentials, scoped key rotation and leases, account identity, endpoint
egress/TLS settings, extra headers/query parameters, temperature policy,
native tool support and applicable provider cache behavior. The compatibility
codecs (`native`, `qnml`, `tagged_json`) wrap the selected model; they do
not provide another credential store or unapproved tool execution path.

Model-facing Requests/Responses streaming is **not equivalent** today:
the pinned Responses adapter performs a unary call and emits a buffered
terminal result rather than incremental Responses SSE. The editor help
must not promise token-by-token streaming for explicit Responses routes.

## Advanced provider editor

Use **Settings > AI > Advanced provider editor** and enter a mapping per line:

```text
vendor/responses-only = responses
vendor/legacy-chat = chat_completions
vendor/default-route = auto
```

The editor saves the mappings via the ordinary Settings/RPC path and keeps
the existing credential authority. The model-selection UI must not interpret
this field as a claim that the provider or account enables that model.

The provider editor's labels, codec options, validation messages and help
text are localized; the dictionaries have locale parity coverage. Saving,
editing, reset and rollback should preserve exact model IDs and modes.

## Acceptance checklist

1. Route one known compatible model over the observed `POST /responses`
   wire path and a sibling over `POST /chat/completions`.
2. Verify that unconfigured model IDs use the original registry route.
3. Reject unsupported auth/transport combinations *before* persisted settings
   or inference operations can expose credentials to the wrong protocol.
4. Verify 401, 403, 429, 5xx, timeout, redirect prevention, cancellation,
   credential lease cleanup and streaming-output no-replay behavior.
5. Exercise QNML/tagged-JSON tool authorization and `ToolChoice::None`
   behavior without executing undeclared tool calls.
6. Test settings serialization and both Rust RPC update surfaces, locale
   parity, provider-editor save/reload, failed-probe rollback and UI
   accessibility at desktop and narrow viewports.
7. Run the full Cargo/frontend/Tauri/hosted CI suites on the **final exact
   commit SHA** before promotion from draft or merging into `main`.

A passing provider `GET /models` listing is not evidence of a model's
Responses, streaming, tool, vision or reasoning capabilities. Automatic
capability inference, full SSE parity, and live remote 2API validation
remain separate future work.
