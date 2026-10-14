//! Contract tests for Jev/Conduit-style source evidence.
use super::SourceEvidenceTool;
use crate::security::policy::{TrustedAccess, TrustedRoot};
use crate::security::SecurityPolicy;
use serde_json::{json, Value};
use std::path::Path;
use std::sync::Arc;
use tinytools::{Tool, ToolCallOptions, ToolRunContext, WorkspaceDescriptor};

fn policy(home: &Path, source: &Path) -> Arc<SecurityPolicy> {
    Arc::new(SecurityPolicy {
        enabled: true,
        workspace_dir: home.to_path_buf(),
        action_dir: source.to_path_buf(),
        workspace_only: true,
        trusted_roots: vec![TrustedRoot {
            path: source.to_string_lossy().to_string(),
            access: TrustedAccess::Read,
        }],
        ..SecurityPolicy::default()
    })
}

async fn run(tool: &SourceEvidenceTool, args: Value) -> Value {
    let output = tool.execute(args).await.expect("source evidence execution");
    assert!(!output.is_error, "{}", output.output());
    serde_json::from_str(output.output()).expect("JSON evidence envelope")
}

#[tokio::test]
async fn scan_has_deterministic_generation_and_line_anchored_evidence() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("b.rs"), "fn beta() { alpha(); }\n").unwrap();
    std::fs::write(project.path().join("a.rs"), "fn alpha() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));

    let first = run(&tool, json!({"operation": "scan", "query": "alpha"})).await;
    let second = run(&tool, json!({"operation": "scan", "query": "alpha"})).await;
    assert_eq!(first["generation"], second["generation"]);
    assert_eq!(first["complete"], true);
    let files = first["files"].as_array().unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files[0]["path"], "a.rs");
    assert_eq!(files[1]["path"], "b.rs");
    assert_eq!(files[0]["matches"][0]["line"], 1);
    assert!(files[0]["source_hash"].as_str().unwrap().len() == 64);

    std::fs::write(project.path().join("a.rs"), "fn alpha_v2() {}\n").unwrap();
    let changed = run(&tool, json!({"operation": "scan", "query": "alpha"})).await;
    assert_ne!(first["generation"], changed["generation"]);
}

#[tokio::test]
async fn verify_rejects_drift_and_never_returns_stale_lines() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let path = project.path().join("lib.rs");
    std::fs::write(&path, "line one\nline two\nline three\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let scan = run(&tool, json!({"operation": "scan"})).await;
    let digest = scan["files"][0]["source_hash"].as_str().unwrap();
    let verified = run(&tool, json!({
        "operation": "verify", "path": "lib.rs",
        "expected_sha256": digest, "start_line": 2, "end_line": 3
    })).await;
    assert_eq!(verified["verified"], true);
    assert_eq!(verified["lines"][0]["line"], 2);
    assert_eq!(verified["lines"][0]["text"], "line two");

    std::fs::write(&path, "edited behind your back\n").unwrap();
    let err = tool.execute(json!({
        "operation": "verify", "path": "lib.rs", "expected_sha256": digest
    })).await.expect_err("stale proof must be refused");
    assert!(err.to_string().contains("changed"));
}

#[tokio::test]
async fn traversal_and_absolute_out_of_scope_paths_are_rejected() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(outside.path().join("outside.rs"), "secret").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    assert!(tool.execute(json!({"operation":"scan","path":"../"})).await.is_err());
    assert!(tool.execute(json!({
        "operation":"verify","path":outside.path().join("outside.rs").to_string_lossy(),
        "expected_sha256":"0".repeat(64)
    })).await.is_err());
}

#[tokio::test]
async fn skips_secret_files_ignored_roots_and_symbolic_links() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("ok.rs"), "visible").unwrap();
    std::fs::write(project.path().join(".env"), "PRIVATE_TOKEN=token").unwrap();
    std::fs::create_dir(project.path().join(".git")).unwrap();
    std::fs::write(project.path().join(".git/config"), "secret git config").unwrap();
    std::fs::write(outside.path().join("secret.rs"), "outside private").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(
        outside.path().join("secret.rs"), project.path().join("link.rs")
    ).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let envelope = run(&tool, json!({"operation":"scan","query":"private"})).await;
    let payload = envelope.to_string();
    assert!(payload.contains("ok.rs"));
    assert!(!payload.contains("PRIVATE_TOKEN"));
    assert!(!payload.contains("outside private"));
    assert!(!payload.contains("secret.rs"));
    assert!(!payload.contains(".git/config"));
    assert!(!payload.contains(".env"));
}

#[tokio::test]
async fn budgets_mark_results_partial_instead_of_claiming_complete() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for index in 0..5 {
        std::fs::write(project.path().join(format!("{index}.rs")), "fn x() {}\n").unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let response = run(&tool, json!({"operation":"scan","max_files":2})).await;
    assert_eq!(response["complete"], false);
    assert_eq!(response["files"].as_array().unwrap().len(), 2);
    assert!(response["truncated_reason"].is_string());
}

struct TestWorkspace(WorkspaceDescriptor);
impl ToolRunContext for TestWorkspace {
    fn workspace(&self) -> Option<&WorkspaceDescriptor> { Some(&self.0) }
}

#[tokio::test]
async fn workspace_descriptor_grants_only_its_root_per_call() {
    let home = tempfile::tempdir().unwrap();
    let ungranted = tempfile::tempdir().unwrap();
    let descriptor_root = tempfile::tempdir().unwrap();
    std::fs::write(descriptor_root.path().join("code.rs"), "fn scoped() {}").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), ungranted.path()));
    let context = TestWorkspace(
        WorkspaceDescriptor::new(descriptor_root.path().to_path_buf())
            .with_policy_id("test-evidence")
    );
    let result = tool.execute_with_context(
        json!({"operation":"scan","path":"."}),
        ToolCallOptions::default(),
        Some(&context)
    ).await.expect("descriptor root authorized");
    assert!(result.output().contains("code.rs"));
    assert!(!result.output().contains(ungranted.path().to_str().unwrap()));
}

#[tokio::test]
async fn verify_requires_a_full_digest_and_bounded_line_range() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("test.rs"), "one\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    assert!(tool.execute(json!({"operation":"verify","path":"test.rs"})).await.is_err());
    assert!(tool.execute(json!({
        "operation":"verify","path":"test.rs","expected_sha256":"nonsense"
    })).await.is_err());
    assert!(tool.execute(json!({
        "operation":"verify","path":"test.rs","expected_sha256":"0".repeat(64),
        "start_line": 80, "end_line": 20
    })).await.is_err());
}


#[tokio::test]
async fn graph_operation_reuses_scoped_read_policy_and_records_source_hashes() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("src/main.ts"), "import './helper';\n").unwrap();
    std::fs::write(project.path().join("src/helper.ts"), "export const ok = true;\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let graph = run(&tool, json!({"operation":"graph","path":"src","focus_path":"src/helper.ts"})).await;
    assert_eq!(graph["operation"], "graph");
    assert_eq!(graph["scan_complete"], true);
    assert_eq!(graph["edges"][0]["status"], "resolved");
    assert_eq!(graph["edges"][0]["to"], "src/helper.ts");
    assert_eq!(graph["impact"], json!(["src/main.ts"]));
    assert_eq!(graph["impact_exhaustive"], false);
    assert_eq!(graph["semantic_graph"], false);
}

#[tokio::test]
async fn graph_never_indexes_credential_files_or_linked_external_source() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("main.ts"), "import './linked';\n").unwrap();
    std::fs::write(project.path().join(".env"), "TOP_SECRET=forbidden").unwrap();
    std::fs::write(outside.path().join("linked.ts"), "export const secret = 123;\n").unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(outside.path().join("linked.ts"), project.path().join("linked.ts")).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let graph = run(&tool, json!({"operation":"graph"})).await;
    assert_eq!(graph["edges"][0]["status"], "unresolved");
    assert!(!graph.to_string().contains("forbidden"));
    assert!(!graph.to_string().contains("linked.ts"));
}

#[tokio::test]
async fn graph_inherits_scan_limits_and_does_not_claim_complete_impact() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for index in 0..5 {
        std::fs::write(project.path().join(format!("{index}.ts")), "export const value = 1;\n").unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let graph = run(&tool, json!({"operation":"graph","max_files":2})).await;
    assert_eq!(graph["scan_complete"], false);
    assert_eq!(graph["edges_complete"], false);
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(graph["impact_exhaustive"], false);
}


#[tokio::test]
async fn scan_pagination_has_stable_order_and_explicit_continuation() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for index in 0..5 {
        std::fs::write(project.path().join(format!("{index}.rs")), format!("fn f{index}() {{}}\n")).unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let first = run(&tool, json!({"operation":"scan","max_files":2})).await;
    assert_eq!(first["files"][0]["path"], "0.rs");
    assert_eq!(first["files"][1]["path"], "1.rs");
    assert_eq!(first["complete"], false);
    assert_eq!(first["next_after_path"], "1.rs");

    let second = run(&tool, json!({
        "operation":"scan","max_files":2,"after_path":first["next_after_path"]
    })).await;
    assert_eq!(second["files"][0]["path"], "2.rs");
    assert_eq!(second["files"][1]["path"], "3.rs");
    assert_eq!(second["next_after_path"], "3.rs");

    let third = run(&tool, json!({
        "operation":"scan","max_files":2,"after_path":second["next_after_path"]
    })).await;
    assert_eq!(third["files"].as_array().unwrap().len(), 1);
    assert_eq!(third["files"][0]["path"], "4.rs");
    assert_eq!(third["complete"], true);
    assert_eq!(third["next_after_path"], Value::Null);
}

#[tokio::test]
async fn traversal_cannot_be_smuggled_as_pagination_marker() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("safe.rs"), "fn safe() {}").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let invalid = tool.execute(json!({
        "operation":"scan","after_path":"../etc/passwd"
    })).await;
    assert!(invalid.is_err());
}
