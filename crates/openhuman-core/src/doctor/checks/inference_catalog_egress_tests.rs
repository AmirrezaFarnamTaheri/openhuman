use super::*;

fn provider(endpoint: String, auth_style: AuthStyle) -> CloudProviderCreds {
    CloudProviderCreds {
        slug: "doctor-egress-test".into(),
        endpoint,
        auth_style,
        ..Default::default()
    }
}

#[tokio::test]
async fn credentialed_catalog_rejects_remote_http_and_url_credentials_before_lookup() {
    let config = crate::config::Config::default();
    for endpoint in [
        "http://198.51.100.1/v1",
        "https://user:secret@example.invalid/v1",
        "https://example.invalid/v1?token=secret",
    ] {
        let entry = provider(endpoint.into(), AuthStyle::Bearer);
        assert!(matches!(
            prepare(&config, &entry).await,
            Err(InspectionCatalogError::Unavailable)
        ));
    }
}

#[tokio::test]
async fn doctor_catalog_does_not_follow_a_redirect_with_an_anthropic_key() {
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let source = MockServer::start().await;
    let destination = MockServer::start().await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&destination)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/models"))
        .and(header("x-api-key", "private-doctor-key"))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("location", format!("{}/models", destination.uri())),
        )
        .expect(1)
        .mount(&source)
        .await;
    let temp = tempfile::tempdir().unwrap();
    let mut config = crate::config::Config {
        config_path: temp.path().join("config.toml"),
        workspace_dir: temp.path().join("workspace"),
        ..Default::default()
    };
    config.secrets.encrypt = false;
    let entry = provider(format!("{}/v1", source.uri()), AuthStyle::Anthropic);
    let auth = crate::security::credentials::AuthService::from_config(&config);
    auth.store_provider_token(
        &crate::inference::provider::factory::auth_key_for_slug(&entry.slug),
        "default",
        "private-doctor-key",
        Default::default(),
        true,
    )
    .unwrap();
    let request = prepare(&config, &entry).await.unwrap();
    assert!(matches!(
        request.fetch().await,
        Err(InspectionCatalogError::Failure)
    ));
}
