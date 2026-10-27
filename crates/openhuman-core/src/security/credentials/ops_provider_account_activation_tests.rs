use super::*;

#[tokio::test]
async fn activating_one_account_only_changes_its_named_cloud_provider() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    for (provider, profile, key, active) in [
        ("provider:openai", "work", "work-secret", true),
        ("provider:openai", "personal", "personal-secret", false),
        ("provider:anthropic", "default", "anthropic-secret", true),
    ] {
        store_provider_credentials(
            &config, provider, Some(profile), Some(key.to_owned()), None, Some(active),
        )
        .await
        .unwrap();
    }

    let auth = AuthService::from_config(&config);
    assert_eq!(
        auth.get_provider_bearer_token("provider:openai", None).unwrap().as_deref(),
        Some("work-secret"),
    );
    let activated = activate_provider_credentials(&config, "provider:openai", "personal")
        .await
        .unwrap();
    assert_eq!(activated.value["provider"], "provider:openai");
    assert_eq!(activated.value["profile"], "personal");
    assert_eq!(activated.value["activeProfileId"], "provider:openai:personal");
    assert!(!serde_json::to_string(&activated.value).unwrap().contains("personal-secret"));
    assert_eq!(
        auth.get_provider_bearer_token("provider:openai", None).unwrap().as_deref(),
        Some("personal-secret"),
    );
    assert_eq!(
        auth.get_provider_bearer_token("provider:anthropic", None).unwrap().as_deref(),
        Some("anthropic-secret"),
        "selecting an account must not alter a different provider",
    );

    let listed = list_provider_credentials(&config, Some("provider:openai".into()))
        .await
        .unwrap();
    assert_eq!(listed.value.len(), 2);
    assert_eq!(
        listed.value.iter().filter(|profile| profile.is_active == Some(true)).count(), 1,
        "exactly one account should be marked preferred",
    );
    assert!(listed.value.iter().find(|profile| profile.profile_name == "personal").unwrap().is_active);
    assert_eq!(listed.value.iter().find(|profile| profile.profile_name == "work").unwrap().is_active, Some(false));
}

#[tokio::test]
async fn missing_or_cross_provider_accounts_cannot_change_active_credentials() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    store_provider_credentials(
        &config, "provider:openai", Some("work"), Some("work-secret".into()), None, Some(true),
    )
    .await
    .unwrap();
    store_provider_credentials(
        &config, "provider:anthropic", Some("other"), Some("other-secret".into()), None,
        Some(true),
    )
    .await
    .unwrap();

    assert!(activate_provider_credentials(&config, "provider:openai", "missing").await.is_err());
    assert!(
        activate_provider_credentials(&config, "provider:openai", "provider:anthropic:other")
            .await.is_err(),
        "a fully qualified cross-provider profile ID must not be selectable",
    );
    let auth = AuthService::from_config(&config);
    assert_eq!(
        auth.get_provider_bearer_token("provider:openai", None).unwrap().as_deref(),
        Some("work-secret"),
    );
}

#[tokio::test]
async fn activation_is_reserved_for_named_cloud_provider_accounts() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    for (provider, profile) in [
        ("app-session", "default"),
        ("api-key", "default"),
        ("channel:telegram", "default"),
        ("provider:", "default"),
        ("", "default"),
        ("provider:openai", ""),
        ("provider:openai", "other:profile"),
    ] {
        assert!(
            activate_provider_credentials(&config, provider, profile).await.is_err(),
            "unsafe account selection should be rejected: {provider}/{profile}",
        );
    }
}

#[tokio::test]
async fn storing_a_non_active_account_does_not_claim_to_make_it_active() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    store_provider_credentials(
        &config, "provider:openai", Some("work"), Some("work-secret".into()), None, Some(true),
    )
    .await
    .unwrap();
    let stored = store_provider_credentials(
        &config, "provider:openai", Some("personal"), Some("personal-secret".into()), None,
        Some(false),
    )
    .await
    .unwrap();
    assert_eq!(stored.value.is_active, None, "write response does not claim active state without listing");
    let listed = list_provider_credentials(&config, Some("provider:openai".into()))
        .await
        .unwrap();
    assert_eq!(listed.value.iter().find(|p| p.profile_name == "work").unwrap().is_active, Some(true));
    assert_eq!(listed.value.iter().find(|p| p.profile_name == "personal").unwrap().is_active, Some(false));
}
