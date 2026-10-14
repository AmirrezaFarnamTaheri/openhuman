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
