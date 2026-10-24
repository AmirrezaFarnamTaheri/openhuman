use super::*;

#[test]
fn profile_revisions_change_only_for_the_saved_store() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let second = root.path().join("second");
    let original = profile_revision(&first);
    profiles_saved(&first);
    let saved = profile_revision(&first).unwrap();
    assert_ne!(Some(saved), original);
    assert_eq!(profile_revision(&first), Some(saved));
    profiles_saved(&second);
    assert_eq!(profile_revision(&first), Some(saved));
    profiles_saved(&first);
    assert_ne!(profile_revision(&first), Some(saved));
}

#[test]
fn failed_profile_persistence_invalidates_potentially_changed_secrets() {
    let root = tempfile::tempdir().unwrap();
    let blocked = root.path().join("blocked");
    std::fs::write(&blocked, b"not a directory").unwrap();
    let mut store = super::super::AuthProfilesStore::new(root.path(), false);
    store.use_keychain = false;
    store.path = blocked.join("auth-profiles.json");
    let original = profile_revision(&blocked);
    assert!(store
        .save_locked(&super::super::AuthProfilesData::default())
        .is_err());
    assert_ne!(profile_revision(&blocked), original);
}

#[test]
fn backend_revision_identity_follows_scope_instead_of_config_directory() {
    let first = Path::new("first-config");
    let second = Path::new("second-config");
    let backend: Arc<dyn crate::storage::StorageBackend> =
        Arc::new(crate::storage::MemoryStorage::new());
    let replacement: Arc<dyn crate::storage::StorageBackend> =
        Arc::new(crate::storage::MemoryStorage::new());
    assert!(
        key_for_scope(first, Some((&backend, "tenant-a")))
            == key_for_scope(second, Some((&backend, "tenant-a")))
    );
    assert!(
        key_for_scope(first, Some((&backend, "tenant-a")))
            != key_for_scope(first, Some((&backend, "tenant-b")))
    );
    assert!(
        key_for_scope(first, Some((&backend, "tenant-a")))
            != key_for_scope(first, Some((&replacement, "tenant-a")))
    );
    assert!(key_for_scope(first, None) != key_for_scope(second, None));
}
