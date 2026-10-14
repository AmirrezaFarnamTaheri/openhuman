use super::*;
#[test]
fn distinct_codex_logins_keep_two_encrypted_profiles_and_route_through_selected_account() {
    use super::super::store::persist_openai_oauth_token_set;
    let tmp = tempdir().unwrap();
    let config = test_config(&tmp);
    let credentials = |bearer: &str| TokenSet {
        access_token: bearer.into(),
        refresh_token: Some(format!("refresh-{bearer}")),
        id_token: None,
        expires_at: Some(Utc::now() + Duration::hours(1)),
        token_type: Some("Bearer".into()),
        scope: None,
    };
    let first = persist_openai_oauth_token_set(
        &config,
        credentials("first-bearer"),
        Some("acct-one".into()),
    )
    .unwrap();
    let second = persist_openai_oauth_token_set(
        &config,
        credentials("second-bearer"),
        Some("acct-two".into()),
    )
    .unwrap();
    assert_ne!(first.id, second.id);
    let data = AuthProfilesStore::new(tmp.path(), false).load().unwrap();
    assert!(data.profiles.contains_key(&first.id));
    assert!(data.profiles.contains_key(&second.id));
    let active = lookup_openai_oauth_credentials(&config).unwrap().unwrap();
    assert_eq!(active.access_token, "second-bearer");
    assert_eq!(active.account_id.as_deref(), Some("acct-two"));
    assert_eq!(
        lookup_key_for_slug("openai", &config).unwrap(),
        "second-bearer"
    );
    // Reconnecting account one updates its own record, not account two.
    let same =
        persist_openai_oauth_token_set(&config, credentials("first-new"), Some("acct-one".into()))
            .unwrap();
    assert_eq!(same.id, first.id);
    assert_eq!(lookup_key_for_slug("openai", &config).unwrap(), "first-new");
    let data = AuthProfilesStore::new(tmp.path(), false).load().unwrap();
    assert_eq!(data.profiles.len(), 2);
    assert_eq!(
        data.profiles
            .get(&second.id)
            .unwrap()
            .token_set
            .as_ref()
            .unwrap()
            .access_token,
        "second-bearer"
    );
    runtime()
        .block_on(disconnect_openai_oauth(&config))
        .unwrap();
    assert!(!data.profiles.is_empty());
    let remaining = AuthProfilesStore::new(tmp.path(), false).load().unwrap();
    assert_eq!(remaining.profiles.len(), 1);
    assert_eq!(
        remaining
            .profiles
            .get(&second.id)
            .unwrap()
            .token_set
            .as_ref()
            .unwrap()
            .access_token,
        "second-bearer"
    );
}

#[test]
fn ambiguous_second_codex_login_cannot_overwrite_a_different_account() {
    use super::super::store::persist_openai_oauth_token_set;
    let tmp = tempdir().unwrap();
    let config = test_config(&tmp);
    let token = |access: &str| TokenSet {
        access_token: access.into(),
        refresh_token: None,
        id_token: None,
        expires_at: None,
        token_type: Some("Bearer".into()),
        scope: None,
    };
    persist_openai_oauth_token_set(&config, token("a"), Some("account-a".into())).unwrap();
    let result = persist_openai_oauth_token_set(&config, token("b"), None);
    assert!(result.unwrap_err().contains("identity"));
    assert_eq!(lookup_key_for_slug("openai", &config).unwrap(), "a");
}
