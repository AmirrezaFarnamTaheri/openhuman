---
description: "Connect OpenHuman to an independent, scoped Prodex Gateway."
icon: network
---

# Prodex Gateway (optional)

OpenHuman and Prodex have complementary execution boundaries. **OpenHuman owns its agent/session, local security, memory, and UI. Prodex Gateway owns its own upstream provider translation, routing policy, quotas, continuation affinity, virtual keys and telemetry.** Connect through the already supported OpenAI-compatible protocol, rather than copying the Prodex account store into OpenHuman.

This integration is **opt-in**. Neither Prodex nor Codex is bundled, launched, installed, or used by default.

## Run the gateway

Install and configure Prodex independently from [christiandoxa/prodex](https://github.com/christiandoxa/prodex). Use its own documented `prodex gateway` command and choose a supported provider. A local illustrative setup is:

```sh
export PRODEX_GATEWAY_CLIENT_KEY='replace-with-a-random-inference-only-token'
export GEMINI_API_KEY='replace-with-your-upstream-provider-token'
# Configure policy.toml (below) and then launch:
prodex gateway --provider gemini
```

For example, set the following in Prodex's own `policy.toml` (and keep the key value in the process environment, not the file):

```toml
[gateway]
listen_addr = "127.0.0.1:4000"
require_auth = true

[[gateway.virtual_keys]]
name = "openhuman-inference"
token_env = "PRODEX_GATEWAY_CLIENT_KEY"
allowed_models = ["prodex-fast"]
request_budget = 1000
rpm_limit = 60
```

**Do not enter `PRODEX_GATEWAY_TOKEN`, an admin token or a Codex OAuth session token into OpenHuman.** An administrator token can access gateway management endpoints; the OpenHuman provider entry needs only a data-plane virtual key with the model/budget/scope restrictions appropriate to the user.

The documented default API root is `http://127.0.0.1:4000/v1`. If the gateway is remote, configure a trusted, authenticated TLS endpoint and edit the corresponding OpenHuman provider URL. `127.0.0.1` always means the host executing the **OpenHuman core**, not necessarily the browser, the phone, or the machine where the gateway was installed.

In **Settings → AI → Add provider**, select **Prodex Gateway**, enter the inference-scoped virtual key, and verify connectivity. OpenHuman discovers models from `/v1/models`. Choose a provider alias or model exposed by Prodex; the route is `prodex:<model-id>` inside OpenHuman's existing routing grammar. If model discovery cannot run, use the standard custom-provider editor and confirm the exact upstream model ID separately.

## Auth and runtime rules

- **Never copy a Codex OAuth token or a Prodex gateway admin token into OpenHuman.** Create a least-privilege inference virtual key.
- A Prodex inference-scoped virtual key is not a direct Anthropic, Gemini or OpenAI API key. OpenHuman sends it as `Authorization: Bearer <token>` to the configured gateway URL; Prodex authenticates it and selects its own upstream profiles.
- Configure a **single gateway key** for this OpenHuman provider entry. Keep provider/account rotation *inside Prodex*. Do not create a second OpenHuman key pool for different gateway identities, as that may defeat continuation affinity and accounting.
- OpenHuman can call `/chat/completions` through its standard compatible adapter. Prodex also supports `/responses`; the built-in endpoint capability gate marks this accurately. A provider's advertised endpoints do not guarantee lossless support for every model capability, attachment, tool shape, or streaming mode.
- Do not interpret this connection as moving OpenHuman sessions, filesystem permissions, billing or access control under Prodex. Only calls explicitly routed to this provider cross the gateway. Verify each requested model/tool/media capability against Prodex's provider matrix.
- In Privacy mode, user policy must still allow outbound requests to the gateway. Loopback routing does not automatically make cloud-backed inference private.
- Admin, governance, SSO and ledger endpoints are **not** called by the provider-preset integration. Use a separate least-privilege administration client for those sensitive operations.

## Validation

Offline checks cover the frontend preset and Rust registry endpoint/auth/capability mapping. A live integration smoke test additionally needs a separately running Prodex instance, an active scoped gateway key, and an available model; the ordinary unit suite cannot assert a real gateway or upstream quota state.

A basic connectivity check is:

```sh
curl -fsS http://127.0.0.1:4000/v1/models \
  -H "Authorization: Bearer ${PRODEX_GATEWAY_CLIENT_KEY}"
```

No actual secrets or HTTP response bodies should be committed in diagnostics.

## Source and attribution

This bridge was derived from the supplied **Prodex v0.437.10** architecture and documented gateway compatibility surface. Prodex source declares **Apache-2.0** in its `Cargo.toml` and `LICENSE`, not MIT. This integration is original glue code; adopting upstream Prodex implementation files would require retaining their applicable notices and license terms.

For developer-owned runtime composition see [OpenAI-compatible providers](openai-compatible.md).