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
    // Inspection reads the selected file, independent of installed mutable storage.
    let provider_key = crate::inference::provider::factory::auth_key_for_slug(&entry.slug);
    let profile_id = format!("{provider_key}:default");
    std::fs::write(
        temp.path().join("auth-profiles.json"),
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "active_profiles": {provider_key.clone(): profile_id.clone()},
            "profiles": {profile_id: {
                "provider": provider_key, "profile_name": "default", "kind": "token",
                "token": "private-doctor-key", "metadata": {}
            }}
        }))
        .unwrap(),
    )
    .unwrap();
    let request = prepare(&config, &entry).await.unwrap();
    assert!(matches!(
        request.fetch().await,
        Err(InspectionCatalogError::Failure)
    ));
}
