use super::*;

#[tokio::test]
async fn can_switch_between_named_accounts_without_rewriting_tokens() {
    let tmp = tempfile::tempdir().unwrap();
    let config = Config {
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    let auth = AuthService::from_config(&config);
    auth.store_provider_token(
        "provider:google",
        "work",
        "work-key",
        Default::default(),
        true,
    )
    .unwrap();
    auth.store_provider_token(
        "provider:google",
        "personal",
        "personal-key",
        Default::default(),
        false,
    )
    .unwrap();
    let result = activate_provider_credentials(&config, "provider:google", "personal")
        .await
        .unwrap();
    assert_eq!(result.value.profile_name, "personal");
    assert!(result.value.is_active);
    assert_eq!(
        auth.get_provider_bearer_token("provider:google", None)
            .unwrap()
            .as_deref(),
        Some("personal-key")
    );
    assert_eq!(
        auth.get_profile("provider:google", Some("work"))
            .unwrap()
            .unwrap()
            .token
            .as_deref(),
        Some("work-key")
    );
    let summaries = list_provider_credentials(&config, Some("provider:google".into()))
        .await
        .unwrap()
        .value;
    assert_eq!(summaries.len(), 2);
    assert_eq!(summaries.iter().filter(|p| p.is_active).count(), 1);
    assert_eq!(
        summaries.iter().find(|p| p.is_active).unwrap().profile_name,
        "personal"
    );
}

#[tokio::test]
async fn refuses_cross_provider_profile_and_app_session_activation() {
    let tmp = tempfile::tempdir().unwrap();
    let config = Config {
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    let auth = AuthService::from_config(&config);
    auth.store_provider_token(
        "provider:anthropic",
        "work",
        "key",
        Default::default(),
        true,
    )
    .unwrap();
    assert!(activate_provider_credentials(
        &config,
        "provider:google",
        "provider:anthropic:work"
    )
    .await
    .is_err());
    assert!(
        activate_provider_credentials(&config, APP_SESSION_PROVIDER, "default")
            .await
            .is_err()
    );
}
