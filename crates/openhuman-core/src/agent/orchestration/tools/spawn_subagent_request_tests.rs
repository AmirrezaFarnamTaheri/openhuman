use super::*;

#[test]
fn async_spawn_decode_rejects_malformed_coding_handoff() {
    let result = decode_async_spawn_args(&serde_json::json!({
        "agent_id": "task_manager_agent",
        "prompt": "investigate",
        "context": "[coding_handoff]\nnot JSON",
    }));
    assert!(result.is_err(), "invalid marked handoff must fail closed");
}

#[tokio::test]
async fn invalid_async_handoff_refused_before_parent_or_registry_lookup() {
    let result = SpawnSubagentTool::new()
        .execute_with_context_inner(
            serde_json::json!({
                "agent_id": "task_manager_agent",
                "prompt": "do not spawn",
                "context": "[coding_handoff]\nnot JSON",
            }),
            None,
            crate::agent::tinyagents::host::OpenHumanRunContext::new(),
            None,
        )
        .await
        .expect("malformed tool input produces a refusal");
    assert!(result.is_error);
    assert!(result.output().contains("spawn_subagent:"));
}
