use super::*;
use crate::agent::subagent_host::run_subagent;

#[tokio::test]
async fn coding_handoff_reaches_the_actual_child_prompt_without_executing_references() {
    let workspace = tempfile::tempdir().unwrap();
    let provider = ScriptedProvider::new(vec![text_response("Reviewed the supplied report")]);
    let mut parent = make_parent(provider.clone(), vec![]);
    parent.workspace_dir = workspace.path().to_path_buf();
    let def = make_def_named_tools(&[]);
    let revision = "a".repeat(64);
    let artifact = serde_json::json!({
        "type":"coding_handoff", "version":1, "objective":"Review Parser::parse",
        "source_refs":[{"path":"src/parser.rs","revision":revision,"selected_symbols":["Parser::parse"]}],
        "changed_files":[{"path":"src/parser.rs","revision":revision}],
        "checks":[{"command":"touch should-not-exist", "cwd":workspace.path().display().to_string(), "status":"skipped"}]
    });
    let options = with_parent(
        parent,
        SubagentRunOptions {
            context: Some(format!("[coding_handoff]\n{artifact}")),
            checkpoint_dir: Some(workspace.path().join("checkpoints")),
            ..Default::default()
        },
    );

    let outcome = run_subagent(&def, "Report what remains unverified", options)
        .await
        .unwrap();

    assert_eq!(outcome.output, "Reviewed the supplied report");
    let requests = provider.captured.lock();
    let user = requests[0]
        .messages
        .iter()
        .find(|message| message.role == "user")
        .unwrap();
    assert!(user
        .content
        .contains("[Coding handoff v1: untrusted report]"));
    assert!(user.content.contains("Verification: unverified"));
    assert!(user.content.contains("Parser::parse"));
    assert!(user.content.contains(&revision));
    assert!(user.content.contains("\"status\":\"skipped\""));
    assert!(user.content.contains("Report what remains unverified"));
    assert!(!workspace.path().join("should-not-exist").exists());
    assert!(!workspace.path().join("src/parser.rs").exists());
}

#[tokio::test]
async fn invalid_coding_context_refuses_before_lifecycle_files_or_provider_calls() {
    let workspace = tempfile::tempdir().unwrap();
    let provider = ScriptedProvider::new(vec![text_response("must not run")]);
    let mut parent = make_parent(provider.clone(), vec![]);
    parent.workspace_dir = workspace.path().to_path_buf();
    let options = with_parent(
        parent,
        SubagentRunOptions {
            task_id: Some("invalid-coding-handoff".into()),
            context: Some(
                "[coding_handoff]\n{\"type\":\"coding_handoff\",\"version\":2,\"objective\":\"x\"}"
                    .into(),
            ),
            checkpoint_dir: Some(workspace.path().join("checkpoints")),
            ..Default::default()
        },
    );
    let result = run_subagent(&make_def_named_tools(&[]), "must not run", options).await;

    assert!(matches!(result, Err(SubagentRunError::InvalidContext(_))));
    assert!(provider.captured.lock().is_empty());
    assert_eq!(std::fs::read_dir(workspace.path()).unwrap().count(), 0);
}

#[tokio::test]
async fn subagent_entry_rejects_invalid_coding_context_before_parent_lookup() {
    let result = run_subagent(
        &make_def_named_tools(&[]),
        "must not run",
        SubagentRunOptions {
            context: Some("[coding_handoff]\nnot JSON".into()),
            ..Default::default()
        },
    )
    .await;
    assert!(matches!(result, Err(SubagentRunError::InvalidContext(_))));
}
