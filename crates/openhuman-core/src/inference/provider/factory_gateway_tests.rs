use super::*;

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
        assert!(
            cloud_slug::prodex_gateway_endpoint_is_safe(endpoint),
            "{endpoint}"
        );
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
                provider
                    .model_image_endpoints
                    .insert("vision-1".into(), url.into());
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
    provider
        .model_image_endpoints
        .insert("vision-1".into(), "http://[::1]:4111/v1".into());
    assert!(cloud_slug::image_destinations_are_safe(&provider));
    provider
        .model_image_endpoints
        .insert("vision-2".into(), "http://remote.example/v1".into());
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

#[test]
fn provider_credential_lookup_failures_do_not_disclose_underlying_secrets() {
    // The credential store or a provider SDK may include sensitive values in
    // a lower-level error. Neither Display nor the Debug/error-chain view may
    // reveal them to an RPC caller, UI, crash report or model.
    let secret = "sk-sensitive-test-only-1234";
    let source = anyhow::anyhow!("backend read failed: Authorization: Bearer {secret}");
    let safe = "[chat-factory] provider credentials are unavailable".to_string();
    let error = super::credentials::redact_credential_lookup_error(source, safe.clone());
    assert_eq!(error.to_string(), safe);
    assert!(!format!("{error:?}").contains(secret));
}
