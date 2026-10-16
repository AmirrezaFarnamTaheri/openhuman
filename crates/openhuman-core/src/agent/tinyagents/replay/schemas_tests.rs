use super::*;
use crate::core::runtime::DispatchCtx;

#[tokio::test]
async fn run_events_rejects_missing_run_id() {
    let err = handle_run_events(DispatchCtx::process_default(), Map::new())
        .await
        .unwrap_err();
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
    let err = handle_run_local_checks(DispatchCtx::process_default(), Map::new())
        .await
        .unwrap_err();
    assert!(err.contains("invalid params"), "{err}");
}

#[test]
fn local_check_run_id_rejects_log_injection_and_path_shaped_inputs() {
    for bad in [
        "",
        "../other",
        "run/foo",
        "run\\foo",
        "run.abc\nspoofed",
        "run.abc\rspoofed",
        "with space",
        "run.\u{0000}",
        "\t",
    ] {
        assert!(
            validate_local_check_run_id(bad).is_err(),
            "run id must not allow path/log control: {bad:?}"
        );
    }
    assert!(validate_local_check_run_id(&"a".repeat(129)).is_err());
    assert!(validate_local_check_run_id("run.deadbeef_123-A").is_ok());
}

#[tokio::test]
async fn local_check_rpc_rejects_bad_run_id_before_workspace_loading() {
    let mut params = Map::new();
    params.insert("run_id".to_owned(), serde_json::json!("../private"));
    let error = handle_run_local_checks(DispatchCtx::process_default(), params)
        .await
        .unwrap_err();
    assert!(error.contains("invalid params"), "{error}");
}
