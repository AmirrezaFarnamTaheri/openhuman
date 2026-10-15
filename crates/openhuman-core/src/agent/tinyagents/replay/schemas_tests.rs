use super::*;

#[tokio::test]
async fn run_events_rejects_missing_run_id() {
    let err = handle_run_events(Map::new()).await.unwrap_err();
    assert!(err.contains("invalid params"), "{err}");
}

#[test]
fn local_check_replay_controller_is_registered_read_only() {
    let names: Vec<_> = all_agent_replay_registered_controllers()
        .into_iter()
        .map(|controller| controller.schema.function)
        .collect();
    assert!(names.contains(&"run_local_checks"));
}

#[tokio::test]
async fn run_local_checks_rejects_missing_run_id_without_accessing_a_store() {
    let err = handle_run_local_checks(Map::new()).await.unwrap_err();
    assert!(err.contains("invalid params"), "{err}");
}
