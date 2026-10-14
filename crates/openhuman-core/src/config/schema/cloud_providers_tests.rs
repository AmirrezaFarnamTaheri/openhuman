//! Tests for the surrounding module.

use super::{
    builtin_cloud_supports_responses_api, endpoint_host, endpoint_host_is_chat_completions_only,
    host_is_builtin_cloud_provider, is_builtin_cloud_slug, is_slug_reserved, AuthStyle,
    BUILTIN_CLOUD_PROVIDERS,
};

#[test]
fn reserved_slugs() {
    for s in ["", " ", "cloud", "openhuman", "pid"] {
        assert!(is_slug_reserved(s), "{s:?} must stay reserved");
    }
}

// Regression: `ollama` was previously reserved, which made the AI settings
// panel unable to persist an `ollama` cloud_providers entry — so the
// model-list dropdown failed with "no cloud provider with id or slug
// 'ollama' found". The factory's chat routing is unaffected by this
// change because the `ollama:<model>` prefix branch fires before any
// cloud_providers lookup.
#[test]
fn ollama_and_lmstudio_are_not_reserved() {
    assert!(
        !is_slug_reserved("ollama"),
        "ollama must be usable as a cloud_providers slug for the /models probe"
    );
    assert!(
        !is_slug_reserved("lmstudio"),
        "lmstudio is a free-form OpenAI-compatible slug"
    );
}

#[test]
fn builtin_cloud_provider_defaults_cover_phase_one_presets() {
    for (slug, label, endpoint, auth_style) in [
        (
            "groq",
            "Groq",
            "https://api.groq.com/openai/v1",
            AuthStyle::Bearer,
        ),
        (
            "deepseek",
            "DeepSeek",
            "https://api.deepseek.com/v1",
            AuthStyle::Bearer,
        ),
        (
            "minimax",
            "MiniMax",
            "https://api.minimax.io/v1",
            AuthStyle::Bearer,
        ),
        (
            "sumopod",
            "SumoPod",
            "https://ai.sumopod.com/v1",
            AuthStyle::Bearer,
        ),
        (
            "modelscope",
            "ModelScope",
            "https://api-inference.modelscope.cn/v1",
            AuthStyle::Bearer,
        ),
    ] {
        let entry = super::builtin_cloud_provider(slug).expect("builtin preset");
        assert_eq!(entry.slug, slug);
        assert_eq!(entry.label, label);
        assert_eq!(entry.endpoint, endpoint);
        assert_eq!(entry.auth_style, auth_style);
    }
}

#[test]
fn builtin_cloud_provider_slugs_are_unique() {
    let mut slugs = std::collections::HashSet::new();
    for provider in BUILTIN_CLOUD_PROVIDERS.iter() {
        assert!(
            slugs.insert(provider.slug),
            "duplicate built-in cloud provider slug {}",
            provider.slug
        );
    }
}

#[test]
fn is_builtin_cloud_slug_matches_presets_only() {
    for slug in ["openai", "deepseek", "groq", "mistral"] {
        assert!(is_builtin_cloud_slug(slug), "{slug} is a built-in preset");
    }
    for slug in ["my-proxy", "custom-openai", "totally-unknown", ""] {
        assert!(
            !is_builtin_cloud_slug(slug),
            "{slug:?} is not a built-in preset"
        );
    }
}

#[test]
fn only_explicit_known_responses_providers_advertise_the_endpoint() {
    assert!(builtin_cloud_supports_responses_api("openai"));
    assert!(builtin_cloud_supports_responses_api("prodex"));
    assert!(builtin_cloud_supports_responses_api("nebius"));
    assert!(builtin_cloud_supports_responses_api("openrouter"));
    assert!(builtin_cloud_supports_responses_api("xai"));
    for slug in ["deepseek", "groq", "mistral", "fireworks", "together"] {
        assert!(
            !builtin_cloud_supports_responses_api(slug),
            "{slug} is chat-completions-only and must not advertise the Responses API"
        );
    }
}

/// Drift guard (TAURI-RUST-5EN): couple the capability helper to the
/// preset list so adding a new built-in that wrongly claims the Responses
/// API — or renaming a known capable provider — fails CI rather than silently
/// re-enabling the guaranteed-404 `/responses` fallback. OpenAI, Prodex,
/// Nebius, OpenRouter and xAI serve `/v1/responses`; the remainder is chat-only unless verified.
#[test]
fn responses_api_capability_is_coupled_to_the_preset_list() {
    for provider in BUILTIN_CLOUD_PROVIDERS.iter() {
        let expected = matches!(provider.slug, "openai" | "prodex" | "nebius" | "openrouter" | "xai");
        assert_eq!(
            builtin_cloud_supports_responses_api(provider.slug),
            expected,
            "built-in {} Responses-API capability drifted from the explicit-capability invariant",
            provider.slug
        );
    }
}

#[test]
fn endpoint_host_parses_scheme_path_and_port() {
    assert_eq!(
        endpoint_host("https://integrate.api.nvidia.com/v1").as_deref(),
        Some("integrate.api.nvidia.com")
    );
    // Missing scheme, mixed case, trailing path.
    assert_eq!(
        endpoint_host("API.OpenAI.com/v1/chat").as_deref(),
        Some("api.openai.com")
    );
    // Userinfo + explicit port are stripped.
    assert_eq!(
        endpoint_host("https://user:pass@api.groq.com:443/openai/v1").as_deref(),
        Some("api.groq.com")
    );
    // Bracketed IPv6 literal with port.
    assert_eq!(
        endpoint_host("http://[::1]:8080/v1").as_deref(),
        Some("::1")
    );
    assert_eq!(endpoint_host("   ").as_deref(), None);
}

/// TAURI-RUST-HW1: the backend-URL resolver uses this to reroute backend
/// domain calls away from a BYO inference host. Every built-in provider host
/// must be recognised; OpenHuman backend hosts and unknown proxies must not.
#[test]
fn host_is_builtin_cloud_provider_recognises_inference_hosts() {
    for host in [
        "openrouter.ai",
        "api.openai.com",
        "api.anthropic.com",
        "api.groq.com",
        "generativelanguage.googleapis.com",
        "API.OPENAI.COM", // case-insensitive
    ] {
        assert!(
            host_is_builtin_cloud_provider(host),
            "{host} is a built-in cloud inference host"
        );
    }
    for host in [
        "api.tinyhumans.ai",
        "staging-api.tinyhumans.ai",
        "my-backend.example",
        "",
    ] {
        assert!(
            !host_is_builtin_cloud_provider(host),
            "{host:?} is not a built-in cloud inference host"
        );
    }
    // Every registry endpoint's own host must classify as builtin.
    for provider in BUILTIN_CLOUD_PROVIDERS.iter() {
        let host = endpoint_host(provider.endpoint).expect("preset endpoint has a host");
        assert!(
            host_is_builtin_cloud_provider(&host),
            "{} ({host}) must be recognised",
            provider.slug
        );
    }
}

/// TAURI-RUST-5A1: a *custom* slug pointed at a known chat-only host (NVIDIA)
/// must be classified chat-only so the factory disables the guaranteed-404
/// `/responses` fallback — the builtin-slug gate alone misses this because
/// the slug is not builtin.
#[test]
fn nvidia_host_is_chat_completions_only_regardless_of_slug() {
    assert!(endpoint_host_is_chat_completions_only(
        "https://integrate.api.nvidia.com/v1"
    ));
    // Other chat-only built-in hosts too.
    for endpoint in [
        "https://api.deepseek.com/v1",
        "https://api.groq.com/openai/v1",
        "https://api.mistral.ai/v1",
    ] {
        assert!(
            endpoint_host_is_chat_completions_only(endpoint),
            "{endpoint} is a chat-completions-only built-in host"
        );
    }
}

#[test]
fn openai_host_and_unknown_proxies_keep_the_responses_fallback() {
    // OpenAI's first-party host serves /responses — must NOT be gated off,
    // even via a custom proxy slug pointed at it.
    assert!(!endpoint_host_is_chat_completions_only(
        "https://api.openai.com/v1"
    ));
    // Genuinely unknown proxy hosts keep the permissive default (they may be
    // real OpenAI proxies that implement /responses).
    for endpoint in [
        "https://my-llm-proxy.internal.example/v1",
        "https://litellm.mycorp.dev/v1",
        "",
    ] {
        assert!(
            !endpoint_host_is_chat_completions_only(endpoint),
            "{endpoint:?} is an unknown host and must keep the fallback"
        );
    }
}

/// Drift guard: the host-based gate must agree with the slug-based
/// capability for every built-in preset's own endpoint, so adding a preset
/// can't silently desync the two gates.
#[test]
fn host_gate_agrees_with_slug_capability_for_every_builtin() {
    for provider in BUILTIN_CLOUD_PROVIDERS.iter() {
        // OpenhumanJwt / Anthropic presets never route through the
        // OpenAI-compatible Responses fallback; the gate only matters for
        // the Bearer OpenAI-compatible hosts.
        if provider.auth_style != AuthStyle::Bearer {
            continue;
        }
        let host_chat_only = endpoint_host_is_chat_completions_only(provider.endpoint);
        let slug_supports = builtin_cloud_supports_responses_api(provider.slug);
        assert_eq!(
            host_chat_only, !slug_supports,
            "host gate for built-in {} disagrees with its slug capability",
            provider.slug
        );
    }
}

#[test]
fn auth_styles_expose_stable_wire_properties() {
    let styles = [
        (AuthStyle::Bearer, "bearer"),
        (AuthStyle::Anthropic, "anthropic"),
        (AuthStyle::OpenhumanJwt, "openhuman_jwt"),
        (AuthStyle::None, "none"),
    ];
    for (style, expected) in styles {
        assert_eq!(style.as_str(), expected);
    }
}

#[test]
fn generated_provider_ids_sanitize_and_bound_the_slug_prefix() {
    let id = super::generate_provider_id("provider with spaces/and_symbols!");
    assert!(id.starts_with("p_provider_with_spaces_"));
    let suffix = id.rsplit('_').next().expect("suffix");
    assert_eq!(suffix.len(), 5);
    assert!(suffix
        .chars()
        .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()));
}

#[test]
fn builtin_table_is_the_upstream_catalog_plus_the_managed_route() {
    use tinyinference_providers::builtin::BUILTIN_CLOUD_PROVIDERS as UPSTREAM;
    let host = &*BUILTIN_CLOUD_PROVIDERS;
    assert_eq!(
        host.len(),
        UPSTREAM.len() + 1 + super::EXTRA_CLOUD_PROVIDERS.len()
    );
    assert_eq!(host[0].slug, "openhuman");
    assert_eq!(host[0].auth_style, AuthStyle::OpenhumanJwt);
    for (mapped, upstream) in host[1..1 + UPSTREAM.len()].iter().zip(UPSTREAM) {
        assert_eq!(mapped.slug, upstream.slug);
        assert_eq!(mapped.label, upstream.label);
        assert_eq!(mapped.endpoint, upstream.endpoint);
        let expected = match upstream.auth_style {
            tinyinference_providers::builtin::BuiltinAuthStyle::Bearer => AuthStyle::Bearer,
            tinyinference_providers::builtin::BuiltinAuthStyle::Anthropic => AuthStyle::Anthropic,
        };
        assert_eq!(mapped.auth_style, expected, "{}", mapped.slug);
    }
}

#[test]
fn per_model_protocol_override_round_trips_without_changing_legacy_defaults() {
    use crate::config::schema::cloud_providers::{CloudProviderCreds, ModelTransport};

    let entry: CloudProviderCreds = serde_json::from_value(serde_json::json!({
        "slug": "multi-protocol-gateway",
        "endpoint": "https://router.example/v1",
        "model_transports": {
            "vendor/response-model": "responses"
        }
    }))
    .expect("explicit model protocol should deserialize");
    assert_eq!(
        entry.model_transport("vendor/response-model"),
        ModelTransport::Responses
    );
    assert_eq!(
        entry.model_transport("vendor/chat-model"),
        ModelTransport::Auto,
        "unlisted models must preserve preexisting registry routing"
    );
    assert_eq!(
        serde_json::to_value(&entry).unwrap()["model_transports"]["vendor/response-model"],
        "responses"
    );

    let legacy: CloudProviderCreds = serde_json::from_value(serde_json::json!({
        "slug": "legacy", "endpoint": "https://legacy.example/v1"
    }))
    .unwrap();
    assert_eq!(legacy.model_transport("any"), ModelTransport::Auto);
    assert!(serde_json::to_value(&legacy)
        .unwrap()
        .get("model_transports")
        .is_none());
    assert!(serde_json::from_value::<CloudProviderCreds>(serde_json::json!({
        "slug": "invalid", "endpoint": "https://example/v1",
        "model_transports": {"model": "unsupported_protocol"}
    }))
    .is_err());
}

#[test]
fn image_endpoint_presets_round_trip_and_default_without_mapping() {
    let entry: super::CloudProviderCreds = serde_json::from_value(serde_json::json!({
        "id": "a",
        "slug": "example",
        "endpoint": "https://api.example.test/v1",
        "image_endpoint": "https://vision.example.test/v1",
        "model_image_endpoints": { "my-vision": "https://models.example.test/v1" }
    }))
    .unwrap();
    assert_eq!(
        entry.image_endpoint.as_deref(),
        Some("https://vision.example.test/v1")
    );
    assert_eq!(
        entry
            .model_image_endpoints
            .get("my-vision")
            .map(String::as_str),
        Some("https://models.example.test/v1")
    );
    let wire = serde_json::to_value(&entry).unwrap();
    assert_eq!(
        wire["model_image_endpoints"]["my-vision"],
        "https://models.example.test/v1"
    );
    let plain: super::CloudProviderCreds = serde_json::from_value(serde_json::json!({
        "slug": "old", "endpoint": "https://api.example.test/v1"
    }))
    .unwrap();
    assert_eq!(plain.image_endpoint, None);
    assert!(plain.model_image_endpoints.is_empty());
}

#[test]
fn reject_invalid_image_endpoints_and_unknown_strategy() {
    use std::collections::HashMap;
    assert!(super::normalize_fleet_config(
        Some("ftp://malicious.example/api".to_string()),
        HashMap::new(),
        None
    )
    .is_err());
    assert!(
        super::normalize_fleet_config(None, HashMap::new(), Some("unknown".to_string())).is_err()
    );
    let mut image_models = HashMap::new();
    image_models.insert(
        "  vision  ".to_owned(),
        " https://vision.example/v1 ".to_owned(),
    );
    let (group, models, strategy) = super::normalize_fleet_config(
        Some(" https://images.example/v1 ".to_owned()),
        image_models,
        Some("least_busy".to_string()),
    )
    .unwrap();
    assert_eq!(group.as_deref(), Some("https://images.example/v1"));
    assert_eq!(
        models.get("vision").map(String::as_str),
        Some("https://vision.example/v1")
    );
    assert_eq!(strategy.as_deref(), Some("least_busy"));
}

#[test]
fn image_model_endpoint_overrides_require_https_outside_loopback() {
    use std::collections::HashMap;

    for url in [
        "http://images.example/v1",
        "http://169.254.169.254/latest/meta-data",
        "http://localhost.bad.example/v1",
        "http://10.0.0.20:8080/v1",
    ] {
        assert!(
            super::normalize_fleet_config(Some(url.to_owned()), HashMap::new(), None).is_err(),
            "unsafe HTTP image group endpoint accepted: {url}"
        );
        let mut overrides = HashMap::new();
        overrides.insert("vision".to_owned(), url.to_owned());
        assert!(
            super::normalize_fleet_config(None, overrides, None).is_err(),
            "unsafe HTTP per-model endpoint accepted: {url}"
        );
    }
    for url in [
        "http://localhost:4000/v1",
        "http://127.0.0.2:4000/v1",
        "http://[::1]:4000/v1",
        "https://images.example/v1",
    ] {
        assert!(
            super::normalize_fleet_config(Some(url.to_owned()), HashMap::new(), None).is_ok(),
            "safe image endpoint rejected: {url}"
        );
    }
}

#[test]
fn qnml_codec_defaults_to_native_and_rejects_unknown_wire_formats() {
    use crate::config::schema::cloud_providers::{CloudProviderCreds, ToolCallCodec};

    let legacy: CloudProviderCreds = serde_json::from_value(serde_json::json!({
        "slug": "existing-gateway",
        "endpoint": "https://gateway.example/v1"
    }))
    .unwrap();
    assert_eq!(legacy.tool_call_codec, ToolCallCodec::Native);
    assert!(
        serde_json::to_value(&legacy)
            .unwrap()
            .get("tool_call_codec")
            .is_none(),
        "a legacy config should not acquire a new serialized default"
    );

    let opted: CloudProviderCreds = serde_json::from_value(serde_json::json!({
        "slug": "qwen-gateway",
        "endpoint": "http://127.0.0.1:4000/v1",
        "tool_call_codec": "qnml"
    }))
    .unwrap();
    assert_eq!(opted.tool_call_codec, ToolCallCodec::Qnml);
    assert_eq!(
        serde_json::to_value(&opted).unwrap()["tool_call_codec"],
        "qnml"
    );

    let unknown: Result<CloudProviderCreds, _> = serde_json::from_value(serde_json::json!({
        "slug": "qwen-gateway",
        "tool_call_codec": "execute_arbitrary_text"
    }));
    assert!(unknown.is_err(), "unsupported tool codecs must fail closed");
}

#[test]
fn additional_fork_cloud_presets_are_represented_exactly_once() {
    for slug in [
        "tokendance",
        "siliconflow",
        "qwen-pay-as-you-go",
        "qwen-token-plan",
    ] {
        let rows: Vec<_> = BUILTIN_CLOUD_PROVIDERS
            .iter()
            .filter(|p| p.slug == slug)
            .collect();
        assert_eq!(rows.len(), 1, "{slug} must have one canonical preset");
        assert_eq!(rows[0].auth_style, AuthStyle::Bearer);
    }
}

#[test]
fn verified_peer_compatible_presets_have_expected_bearer_routes() {
    for (slug, endpoint) in [
        ("aionlabs", "https://api.aionlabs.ai/v1"),
        ("sambanova", "https://api.sambanova.ai/v1"),
        ("ovhcloud", "https://oai.endpoints.kepler.ai.cloud.ovh.net/v1"),
        ("nebius", "https://api.tokenfactory.nebius.com/v1"),
        ("llm7", "https://api.llm7.io/v1"),
        ("synthetic", "https://api.synthetic.new/openai/v1"),
    ] {
        let preset = super::builtin_cloud_provider(slug).expect("verified peer preset");
        assert_eq!(preset.endpoint, endpoint, "{slug} endpoint");
        assert_eq!(preset.auth_style, AuthStyle::Bearer, "{slug} authentication");
        let supports_responses = slug == "nebius";
        assert_eq!(
            builtin_cloud_supports_responses_api(slug),
            supports_responses,
            "{slug} Responses-API capability"
        );
        assert_eq!(
            endpoint_host_is_chat_completions_only(endpoint),
            !supports_responses,
            "{slug} endpoint fallback gate"
        );
    }
}

#[test]
fn llmtr_preset_uses_provider_native_bearer_endpoint() {
    let preset = super::builtin_cloud_provider("llmtr").expect("LLMTR provider preset");
    assert_eq!(preset.label, "LLMTR");
    assert_eq!(preset.endpoint, "https://llmtr.com/v1");
    assert_eq!(preset.auth_style, AuthStyle::Bearer);
}

#[test]
fn prodex_gateway_is_a_builtin_responses_capable_bearer_provider() {
    let rows: Vec<_> = BUILTIN_CLOUD_PROVIDERS
        .iter()
        .filter(|provider| provider.slug == "prodex")
        .collect();
    assert_eq!(rows.len(), 1, "one canonical Prodex entry");
    let preset = rows[0];
    assert_eq!(preset.label, "Prodex Gateway");
    assert_eq!(preset.endpoint, "http://127.0.0.1:4000/v1");
    assert_eq!(preset.auth_style, AuthStyle::Bearer);
    assert!(
        builtin_cloud_supports_responses_api("prodex"),
        "gateway provides /v1/responses"
    );
    assert!(
        !endpoint_host_is_chat_completions_only(preset.endpoint),
        "must not treat gateway as chat-only"
    );
}
