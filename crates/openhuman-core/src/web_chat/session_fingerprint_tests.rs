use super::*;

#[test]
fn persisted_settings_and_runtime_routes_invalidate_cached_sessions() {
    let config = Config::default();
    let original = autonomy_signature(&config);
    let mut changed = config.clone();
    changed.shell.hide_window = !changed.shell.hide_window;
    assert_ne!(autonomy_signature(&changed), original);
    let mut changed = config.clone();
    changed.resolved_endpoints.inference_url = Some("https://example.invalid/v1".into());
    assert_ne!(autonomy_signature(&changed), original);
    let endpoint_changed = autonomy_signature(&changed);
    changed.config_path = changed.config_path.with_file_name("different-config.toml");
    assert_ne!(autonomy_signature(&changed), endpoint_changed);
}

#[test]
fn changing_only_an_ephemeral_credential_invalidates_without_exposing_it() {
    let mut config = Config::default();
    config.ephemeral_route = crate::config::schema::EphemeralRoute::from_params(
        Some("https://example.invalid/v1".into()),
        Some("first-secret".into()),
    );
    let first = autonomy_signature(&config);
    config.ephemeral_route.as_mut().unwrap().api_key = "second-secret".into();
    let second = autonomy_signature(&config);
    assert_ne!(first, second);
    assert_eq!(second.len(), 64);
    assert!(!second.contains("second-secret"));
}

#[test]
fn map_insertion_order_does_not_invalidate_runtime_settings() {
    let mut first = Config::default();
    first.teams.insert("alpha".into(), Default::default());
    first.teams.insert("beta".into(), Default::default());
    let mut second = first.clone();
    second.teams.clear();
    second.teams.insert("beta".into(), Default::default());
    second.teams.insert("alpha".into(), Default::default());
    assert_eq!(autonomy_signature(&first), autonomy_signature(&second));
}

#[test]
fn saved_profile_changes_invalidate_only_the_matching_cached_settings() {
    let root = tempfile::tempdir().unwrap();
    let mut config = Config::default();
    config.config_path = root.path().join("config.toml");
    let original = autonomy_signature(&config);
    let store = crate::security::credentials::profiles::AuthProfilesStore::new(root.path(), false);
    // No real account or secret is touched: this writes an empty temporary store.
    store.clear_active_profile("test-provider").unwrap();
    assert_ne!(autonomy_signature(&config), original);
    let saved = autonomy_signature(&config);
    store.load().unwrap();
    assert_eq!(autonomy_signature(&config), saved);

    let other = tempfile::tempdir().unwrap();
    let other_store =
        crate::security::credentials::profiles::AuthProfilesStore::new(other.path(), false);
    other_store.clear_active_profile("test-provider").unwrap();
    assert_eq!(autonomy_signature(&config), saved);
}
