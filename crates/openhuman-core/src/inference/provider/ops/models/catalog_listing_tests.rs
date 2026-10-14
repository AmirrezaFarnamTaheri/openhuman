use super::*;

#[test]
fn retries_only_auth_rejections_while_untried_credentials_remain() {
    assert!(should_try_next_catalog_key(401, true));
    assert!(should_try_next_catalog_key(403, true));
    assert!(!should_try_next_catalog_key(401, false));
    assert!(!should_try_next_catalog_key(408, true));
    assert!(!should_try_next_catalog_key(429, true));
    assert!(!should_try_next_catalog_key(500, true));
}

#[test]
fn openrouter_validation_tries_another_key_only_for_auth_status() {
    assert!(should_retry_openrouter_key(
        "OpenRouter key validation returned 401: invalid API key",
        true
    ));
    assert!(should_retry_openrouter_key(
        "OpenRouter key validation returned 403: forbidden",
        true
    ));
    assert!(!should_retry_openrouter_key(
        "OpenRouter key validation returned 401: invalid API key",
        false
    ));
    assert!(!should_retry_openrouter_key(
        "OpenRouter key validation failed: connection refused",
        true
    ));
}

#[tokio::test]
async fn candidate_probe_uses_new_key_without_overwriting_existing_profile() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    use crate::security::credentials::AuthService;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("authorization", "Bearer replacement-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [{ "id": "safe-model", "object": "model" }]
        })))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = tempfile::tempdir().unwrap();
    let mut config = crate::config::Config {
        config_path: tmp.path().join("config.toml"),
        workspace_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    config.secrets.encrypt = false;
    config.cloud_providers.push(CloudProviderCreds {
        slug: "probe-candidate".into(),
        endpoint: format!("{}/v1", server.uri()),
        auth_style: AuthStyle::Bearer,
        ..Default::default()
    });
    let auth = AuthService::from_config(&config);
    auth.store_provider_token(
        "provider:probe-candidate",
        "default",
        "previous-key",
        Default::default(),
        true,
    )
    .unwrap();

    let result = list_configured_models_with_candidate_key_from_config(
        "probe-candidate", &config, "replacement-key",
    )
    .await
    .unwrap();
    assert_eq!(result.value["models"][0]["id"], "safe-model");
    assert_eq!(
        auth.get_provider_bearer_token("provider:probe-candidate", None)
            .unwrap()
            .as_deref(),
        Some("previous-key")
    );
}

#[tokio::test]
async fn failed_candidate_probe_preserves_existing_secret() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    use crate::security::credentials::AuthService;
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("authorization", "Bearer invalid-replacement"))
        .respond_with(ResponseTemplate::new(401))
        .expect(1)
        .mount(&server)
        .await;
    let tmp = tempfile::tempdir().unwrap();
    let mut config = crate::config::Config {
        config_path: tmp.path().join("config.toml"),
        workspace_dir: tmp.path().to_path_buf(),
        ..Default::default()
    };
    config.secrets.encrypt = false;
    config.cloud_providers.push(CloudProviderCreds {
        slug: "probe-candidate".into(),
        endpoint: format!("{}/v1", server.uri()),
        auth_style: AuthStyle::Bearer,
        ..Default::default()
    });
    let auth = AuthService::from_config(&config);
    auth.store_provider_token(
        "provider:probe-candidate",
        "default",
        "previous-key",
        Default::default(),
        true,
    )
    .unwrap();

    assert!(
        list_configured_models_with_candidate_key_from_config(
            "probe-candidate", &config, "invalid-replacement",
        )
        .await
        .is_err()
    );
    assert_eq!(
        auth.get_provider_bearer_token("provider:probe-candidate", None)
            .unwrap()
            .as_deref(),
        Some("previous-key")
    );
}

#[tokio::test]
async fn candidate_keys_must_not_be_forwarded_over_plaintext_remote_http() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    let mut config = crate::config::Config::default();
    config.cloud_providers.push(CloudProviderCreds {
        slug: "unsafe-candidate".into(),
        endpoint: "http://example.com/v1".into(),
        auth_style: AuthStyle::Bearer,
        ..Default::default()
    });
    let result = list_configured_models_with_candidate_key_from_config(
        "unsafe-candidate", &config, "secret-that-must-not-leak",
    )
    .await;
    assert!(result.unwrap_err().contains("HTTPS or loopback"));
}

#[tokio::test]
async fn existing_keys_are_never_forwarded_to_remote_plaintext_catalogs() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    let mut config = crate::config::Config::default();
    config.cloud_providers.push(CloudProviderCreds {
        slug: "unsafe-catalog".into(),
        endpoint: "http://example.com/v1".into(),
        auth_style: AuthStyle::Bearer,
        ..Default::default()
    });
    let error = list_configured_models_from_config("unsafe-catalog", &config)
        .await
        .unwrap_err();
    assert!(error.contains("HTTPS or loopback"), "{error}");
}

#[tokio::test]
async fn catalog_probe_refuses_redirects_with_anthropic_keys() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let receiver = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/capture"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "object": "list",
            "data": [{ "id": "wrong-destination", "object": "model" }]
        })))
        .mount(&receiver)
        .await;

    let origin = MockServer::start().await;
    let redirect_url = format!("{}/capture", receiver.uri());
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("x-api-key", "candidate-secret"))
        .respond_with(ResponseTemplate::new(302).insert_header("Location", redirect_url.as_str()))
        .expect(1)
        .mount(&origin)
        .await;

    let mut config = crate::config::Config::default();
    config.cloud_providers.push(CloudProviderCreds {
        slug: "redirecting-anthropic".into(),
        endpoint: format!("{}/v1", origin.uri()),
        auth_style: AuthStyle::Anthropic,
        ..Default::default()
    });

    let result = list_configured_models_with_candidate_key_from_config(
        "redirecting-anthropic", &config, "candidate-secret",
    ).await;
    assert!(result.is_err(), "the catalogue must reject a redirect instead of following it");
    let requests = receiver.received_requests().await.unwrap();
    assert!(requests.is_empty(), "the redirected endpoint must never receive a request");
}

#[test]
fn provider_preview_changes_only_the_supplied_snapshot() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    let mut config = crate::config::Config::default();
    let original = CloudProviderCreds {
        id: "stable-id".into(),
        slug: "previewed".into(),
        endpoint: "https://original.example/v1".into(),
        auth_style: AuthStyle::Bearer,
        ..Default::default()
    };
    config.cloud_providers.push(original.clone());
    let new = CloudProviderCreds {
        endpoint: "https://replacement.example/v1".into(),
        ..original.clone()
    };
    apply_candidate_provider_preview(&mut config, "previewed", new, None).unwrap();
    assert_eq!(config.cloud_providers.len(), 1);
    assert_eq!(config.cloud_providers[0].endpoint, "https://replacement.example/v1");
    assert_eq!(original.endpoint, "https://original.example/v1");
}

#[test]
fn preview_rejects_identity_crossover_and_reserved_provider_ids() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    let mut config = crate::config::Config::default();
    for (slug, id) in [("openhuman", "openhuman"), ("target", "other")] {
        let entry = CloudProviderCreds {
            id: "p_1".into(),
            slug: slug.into(),
            endpoint: "https://models.example/v1".into(),
            auth_style: AuthStyle::Bearer,
            ..Default::default()
        };
        assert!(apply_candidate_provider_preview(&mut config, id, entry, None).is_err());
    }
    assert!(config.cloud_providers.is_empty());
}

#[test]
fn preview_rejects_invalid_ca_without_touching_saved_provider() {
    use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};
    let mut config = crate::config::Config::default();
    let old = CloudProviderCreds {
        slug: "previewed".into(),
        endpoint: "https://original.example/v1".into(),
        auth_style: AuthStyle::Bearer,
        ..Default::default()
    };
    config.cloud_providers.push(old.clone());
    let candidate = CloudProviderCreds {
        endpoint: "https://replacement.example/v1".into(),
        ..old
    };
    assert!(
        apply_candidate_provider_preview(
            &mut config, "previewed", candidate, Some("invalid PEM".into())
        )
        .is_err()
    );
    assert_eq!(config.cloud_providers[0].endpoint, "https://original.example/v1");
}
