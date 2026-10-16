use super::*;
use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
use crate::config::Config;
use crate::security::credentials::AuthService;
use tempfile::TempDir;

use crate::inference::provider::factory::access_gates::host_requires_session;
fn create_test_chat_model_from_string(
    role: &str,
    provider: &str,
    config: &Config,
) -> anyhow::Result<(Arc<dyn ChatModel<()>>, String)> {
    create_chat_model_from_string_with_model_id(role, provider, config, 0.7)
}

fn create_test_local_chat_model_from_string(
    provider: &str,
    config: &Config,
) -> anyhow::Result<(Arc<dyn ChatModel<()>>, String)> {
    create_local_chat_model_from_string(provider, config)
}

fn create_test_omlx_model(
    model: &str,
    _temperature: Option<f64>,
    config: &Config,
) -> anyhow::Result<(Arc<dyn ChatModel<()>>, String)> {
    create_local_chat_model_from_string(&format!("omlx:{model}"), config)
}

fn config_with_providers(providers: Vec<CloudProviderCreds>) -> Config {
    let mut c = Config::default();
    c.cloud_providers = providers;
    c
}

fn config_with_providers_in_tempdir(tmp: &TempDir, providers: Vec<CloudProviderCreds>) -> Config {
    let mut c = config_with_providers(providers);
    c.workspace_dir = tmp.path().join("workspace");
    c.config_path = tmp.path().join("config.toml");
    c
}

fn oh_entry(id: &str) -> CloudProviderCreds {
    CloudProviderCreds {
        id: id.to_string(),
        slug: "openhuman".to_string(),
        label: "OpenHuman".to_string(),
        endpoint: "https://api.openhuman.ai/v1".to_string(),
        auth_style: AuthStyle::OpenhumanJwt,
        ..Default::default()
    }
}

fn openai_entry(id: &str, slug: &str) -> CloudProviderCreds {
    CloudProviderCreds {
        id: id.to_string(),
        slug: slug.to_string(),
        label: "OpenAI".to_string(),
        endpoint: "https://api.openai.com/v1".to_string(),
        auth_style: AuthStyle::Bearer,
        default_model: Some("gpt-4o".to_string()),
        ..Default::default()
    }
}

fn anthropic_entry(id: &str, slug: &str) -> CloudProviderCreds {
    CloudProviderCreds {
        id: id.to_string(),
        slug: slug.to_string(),
        label: "Anthropic".to_string(),
        endpoint: "https://api.anthropic.com/v1".to_string(),
        auth_style: AuthStyle::Anthropic,
        default_model: Some("claude-sonnet-4-6".to_string()),
        ..Default::default()
    }
}

fn nvidia_nim_entry(id: &str, default_model: Option<&str>) -> CloudProviderCreds {
    CloudProviderCreds {
        id: id.to_string(),
        slug: "nvidia-nim".to_string(),
        label: "NVIDIA NIM".to_string(),
        endpoint: "https://integrate.api.nvidia.com/v1".to_string(),
        auth_style: AuthStyle::Bearer,
        default_model: default_model.map(ToString::to_string),
        ..Default::default()
    }
}

// ── config.api_key fallback scoping (PR #2724) ───────────────────────────

/// Build a tempdir-backed Config with a global `config.api_key`, a custom
/// `inference_url`, and two cloud providers: one whose endpoint matches the
/// inference_url (the legacy direct-inference slug) and one that does not.
///
/// The tempdir workspace has no stored auth-profiles, so `lookup_key_for_slug`
/// exhausts the standard auth path and reaches the `config.api_key` fallback.
fn config_for_api_key_fallback(tmp: &TempDir) -> Config {
    let mut custom = openai_entry("p_custom", "custom");
    custom.endpoint = "https://inference.example.com/v1".to_string();
    let config = config_with_providers_in_tempdir(
        tmp,
        vec![custom, anthropic_entry("p_anthropic", "anthropic")],
    );
    let mut config = config;
    config.api_key = Some("global-key".to_string());
    config.inference_url = Some("https://inference.example.com/v1".to_string());
    config
}

// ── #3767: managed-credits gate bypass (gate-only, per-tier) ───────────────
//
// Routing is NOT changed by this fix — selecting a BYO provider already routes
// inference correctly. The gate is evaluated PER TIER so the UI checks whichever
// tier the user actually selected: the chat header's "Quick" mode runs on the
// `chat` tier and "Reasoning" mode on the `reasoning` tier. `role_bypasses_
// managed_credits(role)` is true when that role runs on the user's own funding
// (a BYO cloud key, a local runtime, or claude-code) with usable credentials.
// Tiers that stay managed and run anyway surface the per-call 402 error.

/// Store a usable provider key under the new-style `provider:<slug>` profile so
/// `lookup_key_for_slug` resolves it.
fn store_byo_key(config: &Config, slug: &str, token: &str) {
    let auth = AuthService::from_config(config);
    auth.store_provider_token(
        &format!("provider:{slug}"),
        "default",
        token,
        Default::default(),
        true,
    )
    .expect("store provider token");
}

// ── Motion B (#4727 Phase 3): wire-equivalent BYOK cloud-slug cutover ────────

fn deepseek_entry(id: &str) -> CloudProviderCreds {
    CloudProviderCreds {
        id: id.to_string(),
        slug: "deepseek".to_string(),
        label: "DeepSeek".to_string(),
        endpoint: "https://api.deepseek.com/v1".to_string(),
        auth_style: AuthStyle::Bearer,
        default_model: Some("deepseek-chat".to_string()),
        ..Default::default()
    }
}

// ── Per-call inference route ────────────────────────────────────────────────
//
// These close the loop between `ephemeral_route::apply` and the two resolution
// steps that have to honour it for a routed turn to reach the caller's
// endpoint. Both are read from `Config`, so they are exercised without a
// network or a booted core.

/// A config carrying a resolved model, as `agent_chat` leaves it after applying
/// `model_override` and before building the agent.
fn routed_config(endpoint: &str, api_key: &str, model: &str) -> Config {
    use crate::config::schema::ephemeral_route::{apply, EphemeralRoute};
    let mut config = Config {
        default_model: Some(model.to_string()),
        ..Config::default()
    };
    apply(
        &mut config,
        EphemeralRoute {
            endpoint: endpoint.to_string(),
            api_key: api_key.to_string(),
            headers: Vec::new(),
        },
    );
    config
}

#[test]
fn only_library_hosts_are_exempt_from_app_login() {
    use crate::core::types::HostKind;

    assert!(!host_requires_session(HostKind::Library));
    for host in [HostKind::TauriShell, HostKind::Cli, HostKind::Docker] {
        assert!(host_requires_session(host));
    }
}

#[path = "factory_crate_native_diagnostics_tests.rs"]
mod crate_native_diagnostics_tests;
#[path = "factory_explicit_ctx_tests.rs"]
mod explicit_ctx_tests;
#[path = "factory_crate_native_tests.rs"]
mod crate_native_tests;
#[path = "factory_egress_fallback_tests.rs"]
mod egress_fallback_tests;
#[path = "factory_route_resolution_tests.rs"]
mod route_resolution_tests;

#[test]
fn grouped_catalog_probe_candidates_are_distinct_and_never_a_delimited_bearer() {
    let keys = model_fleet_probe_keys(
        "test-model-fleet-catalog-candidates",
        " bad-key, working-key;bad-key ".to_owned(),
    );
    assert_eq!(keys, vec!["bad-key", "working-key"]);
    assert!(!keys
        .iter()
        .any(|value| value.contains(',') || value.contains(';')));
}

#[test]
fn single_catalog_key_is_preserved_without_an_empty_candidate() {
    assert_eq!(
        model_fleet_probe_keys("test-model-fleet-catalog-single", " only-key ".into()),
        vec!["only-key"],
    );
}

#[test]
fn cloud_fleet_build_uses_resolved_codex_endpoint_without_retargeting_anthropic() {
    use tinyinference_llm::providers::openai::codex::OpenAiCodexRouting;
    let openai = openai_entry("oa", "openai");
    let oauth = OpenAiCodexRouting {
        endpoint: "https://chatgpt.com/backend-api/codex".into(),
        using_oauth: true,
        account_id: Some("acct-1".into()),
    };
    assert_eq!(
        cloud_slug::cloud_base_url(&openai, &oauth),
        "https://chatgpt.com/backend-api/codex"
    );
    let usual = OpenAiCodexRouting::standard("https://api.openai.com/v1");
    assert_eq!(
        cloud_slug::cloud_base_url(&openai, &usual),
        "https://api.openai.com/v1"
    );
    let claude = anthropic_entry("an", "anthropic");
    assert_eq!(
        cloud_slug::cloud_base_url(&claude, &oauth),
        "https://api.anthropic.com/v1"
    );
}

#[test]
fn prodex_gateway_rejects_remote_plaintext_and_embedded_url_credentials() {
    for (url, accepted) in [
        ("http://127.0.0.1:4000/v1", true),
        ("http://localhost:4000/v1", true),
        ("http://[::1]:4000/v1", true),
        ("https://gateway.example.com/v1", true),
        ("http://gateway.example.com/v1", false),
        ("http://127.0.0.1.attacker.example/v1", false),
        ("http://user:password@127.0.0.1:4000/v1", false),
        ("http://127.0.0.1:4000/v1?api_key=secret", false),
        ("https://gateway.example.com/v1#fragment", false),
        ("file:///tmp/gateway", false),
    ] {
        assert_eq!(
            cloud_slug::prodex_gateway_endpoint_is_safe(url),
            accepted,
            "{url} must respect gateway egress security"
        );
    }
    let mut unsafe_provider = openai_entry("p_prodex", "prodex");
    unsafe_provider.endpoint = "http://gateway.example.com/v1".into();
    let config = config_with_providers(vec![unsafe_provider]);
    let error = cloud_slug::resolve_cloud_slug("chat", "prodex", "prodex-fast", &config)
        .err()
        .expect("remote plaintext Prodex gateway must be rejected before auth lookup");
    assert!(error.to_string().contains("HTTPS"), "{error}");
}

#[test]
fn every_credentialed_provider_rejects_insecure_main_endpoints() {
    for endpoint in [
        "http://remote.example/v1",
        "http://169.254.169.254/latest/meta-data",
        "http://127.0.0.1.attacker.example/v1",
        "https://user:password@provider.example/v1",
        "https://provider.example/v1?api_key=secret",
        "file:///tmp/provider",
    ] {
        for slug in ["custom-bearer", "custom-anthropic"] {
            let mut provider = openai_entry("p_secure", slug);
            provider.endpoint = endpoint.into();
            provider.auth_style = if slug == "custom-bearer" {
                AuthStyle::Bearer
            } else {
                AuthStyle::Anthropic
            };
            let config = config_with_providers(vec![provider]);
            let error = cloud_slug::resolve_cloud_slug("chat", slug, "model", &config)
                .err()
                .expect("credentialed endpoint should be rejected before key lookup");
            assert!(error.to_string().contains("HTTPS"), "{endpoint}: {error}");
        }
    }
}

#[test]
fn credentialed_gateway_main_endpoints_allow_https_and_loopback_only() {
    for endpoint in [
        "https://models.example/v1",
        "http://localhost:8000/v1",
        "http://127.0.0.2:8000/v1",
        "http://[::1]:8000/v1",
    ] {
        assert!(cloud_slug::prodex_gateway_endpoint_is_safe(endpoint), "{endpoint}");
    }
}

#[test]
fn prodex_gateway_uses_exactly_one_scoped_virtual_key() {
    assert!(cloud_slug::prodex_has_single_virtual_key(
        "vk-inference-one"
    ));
    assert!(cloud_slug::prodex_has_single_virtual_key(
        "  vk-inference-one  "
    ));
    assert!(!cloud_slug::prodex_has_single_virtual_key(""));
    assert!(!cloud_slug::prodex_has_single_virtual_key("  "));
    assert!(!cloud_slug::prodex_has_single_virtual_key("vk-one,vk-two"));
    assert!(!cloud_slug::prodex_has_single_virtual_key("vk-one;vk-two"));
    assert!(!cloud_slug::prodex_has_single_virtual_key("vk-one\nvk-two"));
}

#[test]
fn prodex_gateway_rejects_unsafe_default_and_model_image_destinations() {
    for url in [
        "http://remote.example/v1",
        "http://127.0.0.1.attacker.example/v1",
        "https://user:secret@example.com/v1",
        "https://example.com/v1?token=secret",
        "file:///tmp/elsewhere",
    ] {
        for model_override in [false, true] {
            let mut entry = openai_entry("p_prodex_images", "prodex");
            entry.endpoint = "http://127.0.0.1:4000/v1".into();
            if model_override {
                entry
                    .model_image_endpoints
                    .insert("unused-image-model".into(), url.into());
            } else {
                entry.image_endpoint = Some(url.into());
            }
            let config = config_with_providers(vec![entry]);
            let err = cloud_slug::resolve_cloud_slug("chat", "prodex", "test-model", &config)
                .err()
                .expect("unsafe image URL must fail before any credential lookup");
            assert!(err.to_string().contains("HTTPS"), "{url}: {err}");
        }
    }
}

#[test]
fn every_cloud_provider_rejects_unsafe_image_routes_loaded_directly_from_config() {
    // UI and RPC validation do not cover a provider entry loaded directly
    // from config.toml. The factory must enforce the same egress policy.
    for url in [
        "http://remote.example/v1",
        "http://169.254.169.254/latest/meta-data",
        "http://127.0.0.1.attacker.example/v1",
        "https://user:secret@images.example/v1",
        "https://images.example/v1?key=secret",
        "file:///tmp/leak",
    ] {
        for by_model in [false, true] {
            let mut provider = openai_entry("p_vision", "vision-cloud");
            if by_model {
                provider.model_image_endpoints.insert("vision-1".into(), url.into());
            } else {
                provider.image_endpoint = Some(url.into());
            }
            let config = config_with_providers(vec![provider]);
            let error =
                cloud_slug::resolve_cloud_slug("chat", "vision-cloud", "text-model", &config)
                    .err()
                    .expect("unsafe image route must fail before any credential lookup");
            assert!(
                error.to_string().contains("HTTPS"),
                "{url} must be rejected for ordinary BYOK providers: {error}"
            );
        }
    }
}

#[test]
fn ordinary_cloud_providers_allow_https_and_loopback_image_routes() {
    let mut provider = openai_entry("p_vision", "vision-cloud");
    provider.image_endpoint = Some("https://images.example/v1".into());
    provider.model_image_endpoints.insert("vision-1".into(), "http://[::1]:4111/v1".into());
    assert!(cloud_slug::image_destinations_are_safe(&provider));
    provider.model_image_endpoints.insert("vision-2".into(), "http://remote.example/v1".into());
    assert!(!cloud_slug::image_destinations_are_safe(&provider));
}

#[test]
fn prodex_gateway_allows_safe_image_destinations() {
    let mut entry = openai_entry("p_prodex_images", "prodex");
    entry.endpoint = "http://127.0.0.1:4000/v1".into();
    entry.image_endpoint = Some("https://images.example.com/v1".into());
    entry
        .model_image_endpoints
        .insert("vision".into(), "http://[::1]:4001/v1".into());
    assert!(
        cloud_slug::prodex_gateway_destinations_are_safe(&entry),
        "TLS and loopback destinations are safe for a scoped gateway key"
    );
    entry
        .model_image_endpoints
        .insert("another-model".into(), "http://remote.example/v1".into());
    assert!(!cloud_slug::prodex_gateway_destinations_are_safe(&entry));
}

#[test]
fn named_token_accounts_join_native_provider_rotation_without_cross_provider_leaks() {
    let tmp = TempDir::new().unwrap();
    let config = config_with_providers_in_tempdir(
        &tmp,
        vec![
            openai_entry("g", "google"),
            anthropic_entry("a", "anthropic"),
        ],
    );
    let auth = AuthService::from_config(&config);
    for (slug, account, key, active) in [
        ("google", "work", "g-key-work", true),
        ("google", "personal", "g-key-personal", false),
        ("anthropic", "work", "anthropic-secret", true),
    ] {
        auth.store_provider_token(
            &format!("provider:{slug}"),
            account,
            key,
            Default::default(),
            active,
        )
        .unwrap();
    }
    let keys =
        super::credentials::lookup_token_account_fleet("google", &config, "g-key-work", false)
            .unwrap();
    assert_eq!(keys, vec!["g-key-work", "g-key-personal"]);
    assert!(!keys.iter().any(|key| key.contains("anthropic")));
    assert_eq!(
        super::credentials::lookup_token_account_fleet(
            "anthropic",
            &config,
            "anthropic-secret",
            false,
        )
        .unwrap(),
        vec!["anthropic-secret"]
    );
}

#[test]
fn named_token_accounts_never_join_oauth_or_delegated_gateway_routes() {
    let tmp = TempDir::new().unwrap();
    let config = config_with_providers_in_tempdir(&tmp, vec![openai_entry("a", "openai")]);
    let auth = AuthService::from_config(&config);
    auth.store_provider_token(
        "provider:openai",
        "personal",
        "api-token",
        Default::default(),
        false,
    )
    .unwrap();
    assert_eq!(
        super::credentials::lookup_token_account_fleet("openai", &config, "oauth-bearer", true,)
            .unwrap(),
        vec!["oauth-bearer"]
    );
    assert_eq!(
        super::credentials::lookup_token_account_fleet(
            "prodex",
            &config,
            "single-virtual-key",
            false,
        )
        .unwrap(),
        vec!["single-virtual-key"]
    );
}

#[test]
fn grouped_legacy_keys_and_named_tokens_are_deduplicated() {
    let tmp = TempDir::new().unwrap();
    let config = config_with_providers_in_tempdir(&tmp, vec![openai_entry("a", "google")]);
    let auth = AuthService::from_config(&config);
    auth.store_provider_token(
        "provider:google",
        "work",
        "first;third;second",
        Default::default(),
        true,
    )
    .unwrap();
    let keys =
        super::credentials::lookup_token_account_fleet("google", &config, "first,second", false)
            .unwrap();
    assert_eq!(keys, vec!["first", "second", "third"]);
}
