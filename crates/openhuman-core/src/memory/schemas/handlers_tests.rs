use super::*;

#[tokio::test]
async fn saving_local_memory_preserves_settings_changed_during_provisioning() {
    let tmp = tempfile::tempdir().unwrap();
    let mut snapshot = crate::memory::test_fixtures::config_in(&tmp);
    snapshot.default_model = Some("before-provisioning".to_string());
    snapshot.save().await.unwrap();
    engine::store_cortexdb_key(&snapshot, "manual-key").unwrap();

    let mut edited = snapshot.clone();
    edited.default_model = Some("chosen-during-provisioning".to_string());
    edited.save().await.unwrap();

    let selected = save_ready_local_engine(&snapshot, "managed-key")
        .await
        .unwrap();
    assert_eq!(selected.default_model, edited.default_model);
    assert_eq!(selected.config_path, snapshot.config_path);
    assert_eq!(selected.workspace_dir, snapshot.workspace_dir);
    assert!(engine::is_managed_cortexdb(&selected));
    assert!(selected.components.memory);
    assert_eq!(
        engine::read_cortexdb_key(&selected).unwrap().as_deref(),
        Some("managed-key")
    );

    let saved = Config::load_from_config_path(&snapshot.config_path, &snapshot.workspace_dir)
        .await
        .unwrap();
    assert_eq!(saved.default_model, edited.default_model);
    assert!(engine::is_managed_cortexdb(&saved));
    assert_eq!(
        engine::read_cortexdb_key(&snapshot).unwrap().as_deref(),
        Some("manual-key")
    );
}
