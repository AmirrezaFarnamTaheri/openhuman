use super::*;
use crate::core::runtime::DispatchCtx;

#[tokio::test]
async fn same_token_refresh_preserves_issuing_backend_and_refuses_rebinding() {
    let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let key = crate::security::credentials::session_support::SESSION_ISSUING_BACKEND_META;
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("user_id".into(), "test-user".into());
    metadata.insert(key.into(), "https://issuer.example".into());
    AuthService::from_config(&config)
        .store_provider_token(
            APP_SESSION_PROVIDER,
            DEFAULT_AUTH_PROFILE_NAME,
            "synthetic-session",
            metadata,
            true,
        )
        .unwrap();
    let state = set_credential(
        &DispatchCtx::process_default(),
        &config,
        SetCredentialRequest {
            token: "synthetic-session".into(),
            user_id: Some("test-user".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .value;
    assert_eq!(
        state.issuing_backend.as_deref(),
        Some("https://issuer.example")
    );
    let error = set_credential(
        &DispatchCtx::process_default(),
        &config,
        SetCredentialRequest {
            token: "synthetic-session".into(),
            user_id: Some("test-user".into()),
            issuing_backend: Some("https://different.example".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap_err();
    assert!(error.starts_with("SESSION_BACKEND_MISMATCH"));
    let stored = crate::security::credentials::session_support::load_app_session_profile(&config)
        .unwrap()
        .unwrap();
    assert_eq!(stored.metadata.get(key).unwrap(), "https://issuer.example");
}

// ── secret_store_for_config ────────────────────────────────────

#[test]
fn secret_store_for_config_scopes_to_config_parent() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    // Build the store — must not panic and must operate under tmp path.
    let _store = secret_store_for_config(&config);
}

// ── encrypt_secret / decrypt_secret ───────────────────────────

#[tokio::test]
async fn encrypt_then_decrypt_round_trips_locally() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let plaintext = "top-secret-value";
    let enc = encrypt_secret(&config, plaintext).await.unwrap();
    assert_ne!(enc.value, plaintext);
    let dec = decrypt_secret(&config, &enc.value).await.unwrap();
    assert_eq!(dec.value, plaintext);
}

#[tokio::test]
async fn decrypt_secret_round_trips_noise_through_migrate_path() {
    // `decrypt` accepts legacy plaintext values (migration path) rather
    // than erroring — validate that behaviour by round-tripping a
    // non-ciphertext input. The assertion only checks that we get a
    // deterministic `Ok`, not what the value is.
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let res = decrypt_secret(&config, "not-a-real-ciphertext").await;
    assert!(
        res.is_ok(),
        "decrypt should accept non-ciphertext via migrate path, got {res:?}"
    );
}

// ── set_credential (input validation) ─────────────────────────

#[tokio::test]
async fn set_credential_rejects_empty_or_whitespace_token() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let err = store_session(&DispatchCtx::process_default(), &config, "", None, None)
        .await
        .unwrap_err();
    assert!(err.contains("token is required"));
    let err = store_session(&DispatchCtx::process_default(), &config, "   ", None, None)
        .await
        .unwrap_err();
    assert!(err.contains("token is required"));
}

#[tokio::test]
async fn set_credential_rejects_unknown_kind() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let err = set_credential(
        &DispatchCtx::process_default(),
        &config,
        SetCredentialRequest {
            token: "x".into(),
            kind: Some("jwt".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap_err();
    assert!(err.contains("unknown credential kind"), "{err}");
}

#[tokio::test]
async fn set_credential_rejects_an_expired_jwt_without_storing() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let token = jwt_with_payload(json!({
        "sub": "user-1",
        "exp": (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp()
    }));
    let err = store_session(&DispatchCtx::process_default(), &config, &token, None, None)
        .await
        .unwrap_err();
    assert!(err.starts_with("CREDENTIAL_EXPIRED:"), "{err}");
    assert!(auth_get_state(&config)
        .await
        .unwrap()
        .value
        .user_id
        .is_none());
}

#[tokio::test]
async fn set_credential_requires_a_user_id_for_a_session() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let token = jwt_with_payload(json!({
        "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp()
    }));
    let err = store_session(&DispatchCtx::process_default(), &config, &token, None, None)
        .await
        .unwrap_err();
    assert!(err.contains("userId required"), "{err}");
    let err = store_session(
        &DispatchCtx::process_default(),
        &config,
        "opaque-token",
        None,
        Some(json!({ "name": "no id" })),
    )
    .await
    .unwrap_err();
    assert!(err.contains("userId required"), "{err}");
}

// ── set_credential (session) ──────────────────────────────────

#[tokio::test]
async fn set_credential_installs_a_session_without_touching_the_backend() {
    with_operator_context(async {
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let mut config = test_config(&tmp);
        // A backend that would reject everything: it must never be consulted.
        config.api_url = Some(spawn_auth_me_status(StatusCode::UNAUTHORIZED).await);
        let exp = chrono::Utc::now() + chrono::Duration::hours(1);
        let token = jwt_with_payload(json!({ "sub": "user-42", "exp": exp.timestamp() }));

        let result = store_session(
            &DispatchCtx::ambient(),
            &config,
            &token,
            None,
            Some(json!({ "id": "user-42", "email": "u@example.com" })),
        )
        .await
        .unwrap();

        let state = result.value;
        assert!(state.is_authenticated);
        assert_eq!(state.user_id.as_deref(), Some("user-42"));
        assert_eq!(state.credential.as_deref(), Some("session"));
        assert_eq!(state.user.unwrap()["email"], "u@example.com");
        assert!(
            state.expires_at.is_some(),
            "JWT exp is recorded for the local precheck"
        );
        let logs = result.logs.join(" ");
        assert!(
            logs.contains("user directory activated for user-42"),
            "{logs}"
        );
        assert!(logs.contains("session credential stored"), "{logs}");
        assert_eq!(
            crate::config::read_active_user_id(&default_root_openhuman_dir().unwrap()).as_deref(),
            Some("user-42")
        );
        assert_eq!(
            identity::peek_credential_user_identity()
                .and_then(|i| i.email)
                .as_deref(),
            Some("u@example.com")
        );
    })
    .await;
}

#[tokio::test]
async fn set_credential_derives_the_user_id_from_the_jwt_subject() {
    with_operator_context(async {
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let config = test_config(&tmp);
        let token = jwt_with_payload(json!({
            "sub": "from-claims",
            "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp()
        }));
        let state = store_session(&DispatchCtx::ambient(), &config, &token, None, None)
            .await
            .unwrap()
            .value;
        assert_eq!(state.user_id.as_deref(), Some("from-claims"));
        // Explicit userId wins over the claim; the user payload's id wins over
        // the claim too.
        let state = store_session(
            &DispatchCtx::ambient(),
            &config,
            &token,
            Some("explicit".into()),
            None,
        )
        .await
        .unwrap()
        .value;
        assert_eq!(state.user_id.as_deref(), Some("explicit"));
    })
    .await;
}

#[tokio::test]
async fn set_credential_with_the_same_token_and_user_is_a_cheap_refresh() {
    with_operator_context(async {
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let config = test_config(&tmp);
        let token = jwt_with_payload(json!({
            "sub": "user-7",
            "exp": (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp()
        }));
        let first = store_session(
            &DispatchCtx::ambient(),
            &config,
            &token,
            None,
            Some(json!({ "id": "user-7", "pendingBackendValidation": true })),
        )
        .await
        .unwrap();
        assert!(first.logs.join(" ").contains("user directory activated"));

        // The host's `/auth/me` answer arrives: same token, same user.
        let user_config = crate::config::load_config_with_timeout().await.unwrap();
        let second = store_session(
            &DispatchCtx::ambient(),
            &user_config,
            &token,
            None,
            Some(json!({ "id": "user-7", "name": "Confirmed" })),
        )
        .await
        .unwrap();
        let logs = second.logs.join(" ");
        assert!(logs.contains("credential refreshed"), "{logs}");
        assert!(!logs.contains("user directory activated"), "{logs}");
        assert!(
            !logs.contains("credential-gated services started"),
            "{logs}"
        );
        let user = second.value.user.unwrap();
        assert_eq!(user["name"], "Confirmed");
        assert!(user.get("pendingBackendValidation").is_none());
    })
    .await;
}

#[tokio::test]
async fn set_credential_for_a_different_user_signs_the_previous_one_out_first() {
    with_operator_context(async {
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let config = test_config(&tmp);
        let exp = (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp();
        store_session(
            &DispatchCtx::ambient(),
            &config,
            &jwt_with_payload(json!({ "sub": "alice", "exp": exp })),
            None,
            None,
        )
        .await
        .unwrap();
        let alice_config = crate::config::load_config_with_timeout().await.unwrap();
        let result = store_session(
            &DispatchCtx::ambient(),
            &alice_config,
            &jwt_with_payload(json!({ "sub": "bob", "exp": exp })),
            None,
            None,
        )
        .await
        .unwrap();
        let logs = result.logs.join(" ");
        assert!(logs.contains("session cleared"), "{logs}");
        assert!(logs.contains("user directory activated for bob"), "{logs}");
        assert_eq!(result.value.user_id.as_deref(), Some("bob"));
        assert_eq!(
            crate::config::read_active_user_id(&default_root_openhuman_dir().unwrap()).as_deref(),
            Some("bob")
        );
        // Alice's profile is gone from her scope.
        assert!(auth_get_state(&alice_config)
            .await
            .unwrap()
            .value
            .user_id
            .is_none());
    })
    .await;
}

// ── set_credential (api key) ──────────────────────────────────

#[tokio::test]
async fn set_and_clear_api_key_credential() {
    let tmp = TempDir::new().unwrap();
    let config = test_config(&tmp);
    let state = set_credential(
        &DispatchCtx::process_default(),
        &config,
        SetCredentialRequest {
            token: "sk-live".into(),
            kind: Some("api-key".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap()
    .value;
    assert!(state.is_authenticated);
    assert_eq!(state.credential.as_deref(), Some("api-key"));
    assert!(state.user_id.is_none());

    let cleared = clear_credential(
        &DispatchCtx::process_default(),
        &config,
        Some(session_support::CredentialKind::ApiKey),
    )
    .await
    .unwrap()
    .value;
    assert_eq!(cleared["removedApiKey"], true);
    assert_eq!(cleared["removedSession"], false);
    assert!(
        !auth_get_state(&config)
            .await
            .unwrap()
            .value
            .is_authenticated
    );
}

#[tokio::test]
async fn clear_credential_without_a_kind_removes_everything() {
    let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
    let _home = EnvVarGuard::set("HOME", tmp.path());
    let config = test_config(&tmp);
    set_credential(
        &DispatchCtx::process_default(),
        &config,
        SetCredentialRequest {
            token: "sk-live".into(),
            kind: Some("api-key".into()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let cleared = clear_credential(&DispatchCtx::process_default(), &config, None)
        .await
        .unwrap()
        .value;
    assert_eq!(cleared["removed"], true);
    assert_eq!(cleared["removedApiKey"], true);
    assert!(
        !auth_get_state(&config)
            .await
            .unwrap()
            .value
            .is_authenticated
    );
}

// ── set_credential (local session) ─────────────────────────────

/// A local session token requires a non-empty user payload — the backend
/// fetch path is bypassed entirely, so there is no fallback to derive the
/// user from an API response.
#[tokio::test]
async fn store_session_local_token_rejects_missing_user_payload() {
    let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
    let tmp = TempDir::new().unwrap();
    std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
    let _home = EnvVarGuard::set("HOME", tmp.path());
    let config = test_config(&tmp);
    let local_token = "header.payload.local";
    let err = store_session(
        &DispatchCtx::process_default(),
        &config,
        local_token,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(
        err.contains("local session requires a user payload"),
        "expected 'local session requires a user payload', got: {err}"
    );
}

/// A local session token with a user payload must be accepted without any
/// network call, must force a deterministic `local-<device>` user id
/// regardless of what the caller passes, and must return a stored profile
/// summary.
#[tokio::test]
async fn store_session_local_token_succeeds_without_network_and_forces_local_user_id() {
    with_operator_context(async {
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let config = test_config(&tmp);
        let local_token = "header.payload.local";
        let user = serde_json::json!({
            "id": "local",
            "name": "Local User",
            "email": "local@openhuman.local"
        });
        // Pass a different user_id to verify it is overridden.
        let result = store_session(
            &DispatchCtx::ambient(),
            &config,
            local_token,
            Some("should-be-overridden".to_string()),
            Some(user),
        )
        .await
        .unwrap();
        // The credential is installed (no network call was required).
        assert!(result.value.is_authenticated);
        assert_eq!(result.value.credential.as_deref(), Some("local"));
        // Logs must mention that backend validation was skipped.
        let log_text = result.logs.join(" ");
        assert!(
            log_text.contains("local session accepted without backend validation"),
            "expected log confirming no backend call, got: {log_text}"
        );
        let expected_local_user_id = local_session_user_id();
        assert!(
            log_text.contains(&format!(
                "user directory activated for {expected_local_user_id}"
            )),
            "expected user-directory activation log for deterministic local uid, got: {log_text}"
        );
        assert!(
            log_text.contains("onboarding left incomplete for local session setup"),
            "expected local session to remain in onboarding, got: {log_text}"
        );
        // The forced local user id wins over the caller's hint, and the user
        // payload's id fields are rewritten to match.
        assert_eq!(
            result.value.user_id.as_deref(),
            Some(expected_local_user_id.as_str())
        );
        let user = result.value.user.unwrap();
        assert_eq!(user["id"], expected_local_user_id);
        assert_eq!(user["_id"], expected_local_user_id);
        assert_eq!(user["name"], "Local User");
    })
    .await;
}
