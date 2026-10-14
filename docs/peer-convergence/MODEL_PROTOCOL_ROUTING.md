# Per-model protocol routing — OpenHuman convergence

This is the first transport-level slice of the P0 capability-routing work in
`API_PEERS_SECOND_ORDER.md`. It is **not** automatic protocol discovery.

## Contract

`cloud_providers[].model_transports` is a map keyed by the exact upstream
model id (not a prefix, model family, provider slug or friendly display name):

```json
{
  "slug": "mixed-gateway",
  "endpoint": "https://gateway.example/v1",
  "auth_style": "bearer",
  "model_transports": {
    "vendor/response-only-model": "responses"
  }
}
```

- Absent or `auto` uses the existing upstream ProviderRegistry implementation,
  including native Anthropic Messages where it is already selected, and Codex
  OAuth's Responses shape.
- `responses` chooses the pinned native OpenAI Responses adapter at
  `{endpoint}/responses` for **only that concrete model**; it does not silently
  switch siblings or retry an unrelated model under a different protocol.
- Anthropic API-key authentication cannot be paired with the opt-in OpenAI
  Responses transport. Incompatible configurations fail rather than sending
  credentials to an incorrect protocol path.
- The original scoped provider credentials, configured TLS/proxy client,
  temperature handling, native-tool metadata, static headers and query
  parameters remain attached to the model.
- The QNML/tagged-JSON compatibility adapters, if enabled, still wrap the
  selected native model; they do not become an alternative credential authority.

## UI

Settings → Advanced provider editor → Model-specific API protocols.
Enter one exact model id and `responses` per line:

```text
vendor/response-only-model = responses
vendor/legacy-chat-model = auto
```

The parser rejects duplicate ids, control characters, missing modes,
unsupported transport strings and more than 512 entries. The persisted record
has no secret values; existing provider secrets remain in AuthService.

## Rollout and safety

A static endpoint can serve different models with different protocol support.
No capability is inferred from the provider's name alone. An unknown model
remains `auto`, not `responses`. Provider model catalogs may be incomplete,
and a successful `GET /models` is not proof that a model supports the
Responses API, structured tool calls, reasoning, images or long-running jobs.

Before production rollout, test with disposable accounts:
1. Existing chat-only models continue POSTing `/chat/completions`.
2. Exactly mapped models POST `/responses` on the configured endpoint.
3. Invalid auth/protocol combinations fail without emitting a request.
4. Streamed text/tools/usage and refusal paths preserve native events.
5. Provider-key 401/429 cooldown, stream no-replay, cancellation and
   credential-lease lifecycle continue to apply.
6. The UI preserves overrides through editing, failed-verification rollback,
   save, reload and explicit reset.

## Next convergence boundary

A versioned, provenance-aware model capability record with live discovery and
expiry is still needed for automatic protocol choices. It must distinguish
provider/model/endpoint, modal inputs, tools, reasoning and streaming support.
Never infer that a whole provider supports `/responses` from one model.
