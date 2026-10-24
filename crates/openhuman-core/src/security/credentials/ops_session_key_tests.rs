//! Session-only sign-out preserves the active API key across user scopes.

use super::*;
use crate::core::runtime::DispatchCtx;
use crate::security::credentials::api_key;

// #6318 — a user with both a session and an API key must keep the key
// usable after the session-only sign-out deactivates the user-scoped
// directory the key was stored beside.
#[tokio::test]
async fn clearing_the_session_preserves_a_coexisting_api_key() {
    with_operator_context(async {
        // Reaches the connector module, whose route is process-wide: hold the module
        // lock (before TEST_ENV_LOCK, the established order) for the whole test.
        let _module = crate::integrations::composio::module_client::module_guard().await;
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let config = test_config(&tmp);

        // Install the session first: this activates the user-scoped directory.
        let exp = chrono::Utc::now() + chrono::Duration::hours(1);
        let token = jwt_with_payload(json!({ "sub": "user-42", "exp": exp.timestamp() }));
        store_session(
            &DispatchCtx::ambient(),
            &config,
            &token,
            None,
            Some(json!({ "id": "user-42" })),
        )
        .await
        .unwrap();

        // Store the API key against the now-active user-scoped config, exactly
        // as the dispatcher would for a follow-up `auth.set_credential` call.
        let user_scoped = crate::config::load_config_with_timeout().await.unwrap();
        set_credential(
            &DispatchCtx::ambient(),
            &user_scoped,
            SetCredentialRequest {
                token: "sk-live".into(),
                kind: Some("api-key".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(
            api_key::has_api_key(&user_scoped),
            "api key must be stored beside the user-scoped config before sign-out"
        );

        // Sign the session out only — the API key must not be cleared.
        let cleared = clear_credential(
            &DispatchCtx::ambient(),
            &user_scoped,
            Some(session_support::CredentialKind::Session),
        )
        .await
        .unwrap()
        .value;
        assert_eq!(cleared["removedSession"], true);
        assert_eq!(cleared["removedApiKey"], false);

        // The process is now back on the pre-login/signed-out config. The key
        // must be readable — and `auth.get_state` must report it — from there,
        // not stranded under the deactivated user directory.
        let signed_out = crate::config::load_config_with_timeout().await.unwrap();
        assert_ne!(
            signed_out.config_path, user_scoped.config_path,
            "sign-out must have rebound to a different (pre-login) config"
        );
        assert!(
            api_key::has_api_key(&signed_out),
            "the API key must survive under the post sign-out config"
        );
        let state = auth_get_state(&signed_out).await.unwrap().value;
        assert!(
            state.is_authenticated,
            "auth.get_state must see the preserved api key after session sign-out"
        );
        assert_eq!(state.credential.as_deref(), Some("api-key"));

        // The key must have moved, not been copied: the deactivated user-scoped
        // location must no longer carry it, or clearing the copy (or logging
        // back into this user) would resurrect a duplicate (#6318 review follow-up).
        assert!(
            !api_key::has_api_key(&user_scoped),
            "the source user-scoped location must no longer hold the api key after it was carried forward"
        );
    }).await;
}

// #6318 (review follow-up) — a stale key already sitting at the pre-login
// workspace must never outrank the key that was actually the active
// credential a moment ago. This RPC only removes the session; it must not
// let an unrelated leftover key silently become the effective one.
#[tokio::test]
async fn clearing_the_session_preserves_the_active_key_over_a_stale_destination_key() {
    with_operator_context(async {
        // Reaches the connector module, whose route is process-wide: hold the module
        // lock (before TEST_ENV_LOCK, the established order) for the whole test.
        let _module = crate::integrations::composio::module_client::module_guard().await;
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        // `test_config` also binds the memory diagnostics `store_session` below
        // needs; its own config_path is a plain tmp fixture unrelated to the real
        // pre-login layout, so it is not where the RPC dispatcher would actually
        // read/write the pre-login api-key profile from.
        let _diagnostics = test_config(&tmp);

        // Key A already sits at the real pre-login/signed-out scope — e.g. left
        // over from an earlier api-key-only run before any session existed.
        let pre_login = crate::config::load_config_with_timeout().await.unwrap();
        set_credential(
            &DispatchCtx::ambient(),
            &pre_login,
            SetCredentialRequest {
                token: "sk-stale-a".into(),
                kind: Some("api-key".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            api_key::get_api_key(&pre_login).unwrap().as_deref(),
            Some("sk-stale-a")
        );

        // Install a session: this activates the user-scoped directory, distinct
        // from the pre-login one key A lives beside.
        let exp = chrono::Utc::now() + chrono::Duration::hours(1);
        let token = jwt_with_payload(json!({ "sub": "user-99", "exp": exp.timestamp() }));
        store_session(
            &DispatchCtx::ambient(),
            &pre_login,
            &token,
            None,
            Some(json!({ "id": "user-99" })),
        )
        .await
        .unwrap();

        // Key B is the one actually active while the session is up.
        let user_scoped = crate::config::load_config_with_timeout().await.unwrap();
        set_credential(
            &DispatchCtx::ambient(),
            &user_scoped,
            SetCredentialRequest {
                token: "sk-live-b".into(),
                kind: Some("api-key".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert_eq!(
            api_key::get_api_key(&user_scoped).unwrap().as_deref(),
            Some("sk-live-b")
        );

        // A session-only clear must preserve key B as the effective credential,
        // not let the stale key A at the destination win by default.
        clear_credential(
            &DispatchCtx::ambient(),
            &user_scoped,
            Some(session_support::CredentialKind::Session),
        )
        .await
        .unwrap();

        let signed_out = crate::config::load_config_with_timeout().await.unwrap();
        assert_eq!(
            api_key::get_api_key(&signed_out).unwrap().as_deref(),
            Some("sk-live-b"),
            "the key that was actually active before sign-out must remain effective, \
             not a stale key that happened to already sit at the destination"
        );
    })
    .await;
}

// #6318 (review follow-up) — `clear_credential(None)` promises to remove
// every credential. The API key was stored beside the user-scoped config
// while the session was active; the session teardown rebinds every process
// global to the pre-login config before the api-key clear runs, so clearing
// against that new location alone would miss the key at its real, original
// location and leave it clearable/resurrectable later.
#[tokio::test]
async fn clearing_without_a_kind_removes_a_user_scoped_api_key_at_its_source() {
    with_operator_context(async {
        // Reaches the connector module, whose route is process-wide: hold the module
        // lock (before TEST_ENV_LOCK, the established order) for the whole test.
        let _module = crate::integrations::composio::module_client::module_guard().await;
        let _env_guard = crate::config::TEST_ENV_LOCK.lock().await;
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("workspace")).unwrap();
        let _home = EnvVarGuard::set("HOME", tmp.path()).without("OPENHUMAN_WORKSPACE");
        let config = test_config(&tmp);

        let exp = chrono::Utc::now() + chrono::Duration::hours(1);
        let token = jwt_with_payload(json!({ "sub": "user-77", "exp": exp.timestamp() }));
        store_session(
            &DispatchCtx::ambient(),
            &config,
            &token,
            None,
            Some(json!({ "id": "user-77" })),
        )
        .await
        .unwrap();

        let user_scoped = crate::config::load_config_with_timeout().await.unwrap();
        set_credential(
            &DispatchCtx::ambient(),
            &user_scoped,
            SetCredentialRequest {
                token: "sk-live".into(),
                kind: Some("api-key".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();
        assert!(api_key::has_api_key(&user_scoped));

        let cleared = clear_credential(&DispatchCtx::ambient(), &user_scoped, None)
            .await
            .unwrap()
            .value;
        assert_eq!(cleared["removed"], true);
        assert_eq!(cleared["removedSession"], true);
        assert_eq!(
            cleared["removedApiKey"], true,
            "clearing everything must report the api key as removed even though it lived \
             beside the now-deactivated user-scoped config"
        );

        assert!(
            !api_key::has_api_key(&user_scoped),
            "the api key must be gone from its real (user-scoped) location, not just the \
             post-teardown config clear_credential ends up using"
        );
        let signed_out = crate::config::load_config_with_timeout().await.unwrap();
        assert!(!api_key::has_api_key(&signed_out));
        assert!(
            !auth_get_state(&signed_out)
                .await
                .unwrap()
                .value
                .is_authenticated
        );
    })
    .await;
}
