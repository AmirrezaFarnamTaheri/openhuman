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


#[tokio::test]
async fn non_utf8_source_is_not_silently_reported_as_complete() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("binary.rs"), [0xff, 0xfe, 0x00]).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan"})).await;
    assert_eq!(result["complete"], false);
    assert_eq!(result["files"].as_array().unwrap().len(), 0);
    assert_eq!(result["truncated_reason"], "unreadable_or_unsupported_file");
}

#[tokio::test]
async fn oversized_source_is_reported_as_partial_without_allocating_file_contents() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("huge.rs"), vec![b'x'; 256 * 1024 + 1]).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan"})).await;
    assert_eq!(result["complete"], false);
    assert_eq!(result["truncated_reason"], "oversize_file");
    assert_eq!(result["files"].as_array().unwrap().len(), 0);
}


#[tokio::test]
async fn global_path_traversal_budget_bounds_sparse_directory_trees() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for index in 0..12 {
        std::fs::create_dir(project.path().join(format!("sub{index}"))).unwrap();
    }
    std::fs::write(project.path().join("root.rs"), "fn root() {}").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan","max_entries":3})).await;
    assert_eq!(result["complete"], false);
    assert_eq!(result["truncated_reason"], "max_entries");
    assert!(result["files"].as_array().unwrap().len() <= 2);
}


#[tokio::test]
async fn scan_serializes_large_escaped_matches_without_corrupting_json() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let line = format!("MATCH {}\n", "\u{0007}".repeat(155));
    for index in 0..20 {
        std::fs::write(project.path().join(format!("{index:02}.rs")), line.repeat(3)).unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let output = tool.execute(json!({"operation":"scan","query":"MATCH"})).await.unwrap();
    assert!(!output.is_error);
    let raw = output.output();
    assert!(raw.len() <= 26_000, "oversized JSON result: {}", raw.len());
    let result: Value = serde_json::from_str(raw).expect("valid bounded JSON");
    assert_eq!(result["complete"], false);
    assert_eq!(result["truncated_reason"], "max_response_bytes");
}

#[tokio::test]
async fn verify_serializes_long_escaped_lines_with_resume_cursor() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let line = format!("MATCH {}\n", "\u{0007}".repeat(155));
    std::fs::write(project.path().join("file.rs"), line.repeat(80)).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let scan = run(&tool, json!({"operation":"scan"})).await;
    let sha = scan["files"][0]["source_hash"].as_str().unwrap();
    let output = run(&tool, json!({
        "operation":"verify","path":"file.rs","expected_sha256":sha,
        "start_line":1,"end_line":80
    })).await;
    assert!(output.to_string().len() <= 26_000);
    assert_eq!(output["verified"], true);
    assert_eq!(output["lines_complete"], false);
    let last = output["lines"].as_array().unwrap().last().unwrap()["line"].as_u64().unwrap();
    assert_eq!(output["next_line"], last + 1);
}

#[tokio::test]
async fn partial_catalog_must_not_confirm_unique_dependencies_or_impact() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("a.ts"), "import './b';\n").unwrap();
    std::fs::write(project.path().join("b.ts"), "export const b = 1;\n").unwrap();
    std::fs::write(project.path().join("c.ts"), "export const c = 1;\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({
        "operation":"graph","max_files":2,"focus_path":"b.ts"
    })).await;
    assert_eq!(result["scan_complete"], false);
    assert_eq!(result["edges"][0]["status"], "catalog_incomplete");
    assert_eq!(result["edges"][0]["observed_candidate"], "b.ts");
    assert!(result["edges"][0]["to"].is_null());
    assert_eq!(result["impact"], json!([]));
    assert_eq!(result["impact_omitted_reason"], "incomplete_catalog");
}

#[cfg(unix)]
#[tokio::test]
async fn explicitly_addressed_symlink_file_cannot_bypass_scan_exclusion() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("target.rs"), "fn target() {}\n").unwrap();
    std::os::unix::fs::symlink(project.path().join("target.rs"), project.path().join("link.rs")).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    assert!(tool.execute(json!({"operation":"scan","path":"link.rs"})).await.is_err());
}


#[tokio::test]
async fn scoped_graph_reports_projection_scope_without_claiming_repository_coverage() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("pkg")).unwrap();
    std::fs::write(project.path().join("pkg/main.ts"), "export const local = 1;\n").unwrap();
    std::fs::write(project.path().join("outside.ts"), "export const external = 1;\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"graph","path":"pkg"})).await;
    assert_eq!(result["scan_complete"], true);
    assert_eq!(result["scope_path"], "pkg");
    assert_eq!(result["scope_is_workspace"], false);
    assert_eq!(result["impact_exhaustive"], false);
    assert_eq!(result["nodes"].as_array().unwrap().len(), 1);
}


#[tokio::test]
async fn explicit_sensitive_ancestor_paths_are_excluded_even_when_file_basename_is_safe() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join(".git")).unwrap();
    std::fs::write(project.path().join(".git/config"), "fake-token = never-return").unwrap();
    std::fs::create_dir(project.path().join("node_modules")).unwrap();
    std::fs::write(project.path().join("node_modules/notes.rs"), "fake-token = never-return").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    for requested in [".git/config", "node_modules/notes.rs"] {
        assert!(tool.execute(json!({"operation":"scan","path":requested})).await.is_err(),
            "must reject explicitly targeted excluded ancestor {requested}");
        assert!(tool.execute(json!({
            "operation":"verify", "path":requested,
            "expected_sha256":"0".repeat(64)
        })).await.is_err());
    }
}

#[tokio::test]
async fn paginated_graph_never_marks_last_page_as_complete_repository_catalog() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("a.ts"), "import './z';\n").unwrap();
    std::fs::write(project.path().join("z.ts"), "export const z = 1;\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let graph = run(&tool, json!({
        "operation":"graph","after_path":"a.ts","focus_path":"z.ts"
    })).await;
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 1);
    assert_eq!(graph["catalog_complete"], false);
    assert_eq!(graph["impact"], json!([]));
    assert_eq!(graph["impact_omitted_reason"], "incomplete_catalog");
    assert_eq!(graph["scan_complete"], false);
    assert_eq!(graph["after_path"], "a.ts");
}

#[tokio::test]
async fn byte_budget_partial_scan_supplies_resume_cursor() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let body = "a".repeat(255 * 1024);
    for i in 0..12 {
        std::fs::write(project.path().join(format!("{i:02}.rs")), &body).unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let first = run(&tool, json!({"operation":"scan"})).await;
    assert_eq!(first["complete"], false);
    assert_eq!(first["truncated_reason"], "max_total_bytes");
    let cursor = first["next_after_path"].as_str().expect("byte budget cursor");
    assert_eq!(cursor, "07.rs");
    let second = run(&tool, json!({"operation":"scan","after_path":cursor})).await;
    assert_eq!(second["files"][0]["path"], "08.rs");
}


#[tokio::test]
async fn sensitive_directory_exclusion_is_case_insensitive_and_blocks_nested_files() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for directory in [".GIT", "NODE_MODULES", ".CACHE-build"] {
        let folder = project.path().join(directory);
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("payload.rs"), "fn private_code() {}").unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let root_scan = run(&tool, json!({"operation":"scan","path":"."})).await;
    assert_eq!(root_scan["files"].as_array().unwrap().len(), 0);
    for directory in [".GIT", "NODE_MODULES", ".CACHE-build"] {
        let target = format!("{directory}/payload.rs");
        assert!(tool.execute(json!({"operation":"scan","path":target})).await.is_err(),
            "case-insensitive deny boundary must reject {target}");
    }
}

#[tokio::test]
async fn hidden_cloud_and_machine_credential_directories_are_never_indexed() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for directory in [".ssh", ".aws", ".kube", ".azure", ".gnupg", ".docker"] {
        let folder = project.path().join(directory);
        std::fs::create_dir(&folder).unwrap();
        std::fs::write(folder.join("config"), "DO_NOT_SURFACE_CREDENTIAL").unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan","query":"DO_NOT_SURFACE"})).await;
    assert_eq!(result["files"].as_array().unwrap().len(), 0);
    for directory in [".ssh", ".aws", ".kube", ".azure", ".gnupg", ".docker"] {
        let candidate = format!("{directory}/config");
        assert!(tool.execute(json!({"operation":"scan","path":candidate})).await.is_err(),
            "must reject explicit machine-credential directory: {candidate}");
    }
}


#[tokio::test]
async fn locate_ranks_hash_anchored_source_lines_without_a_remote_model() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("src/session_manager.ts"),
        "export function assertSessionFresh(s: Session) { return s.expiresAt > Date.now(); }\n").unwrap();
    std::fs::write(project.path().join("src/billing.ts"),
        "export function chargeInvoice() { return 1; }\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"locate","query":"session freshness","path":"."})).await;
    assert_eq!(result["operation"], "locate");
    assert_eq!(result["results"][0]["path"], "src/session_manager.ts");
    assert_eq!(result["results"][0]["evidence"][0]["line"], 1);
    assert_eq!(result["results"][0]["source_hash"].as_str().unwrap().len(), 64);
    assert_eq!(result["ranking_kind"], "local_lexical_heuristic");
    assert_eq!(result["source_of_truth_verified"], false);
    assert_eq!(result["catalog_complete"], true);
}

#[tokio::test]
async fn locate_is_explicitly_partial_when_file_budget_or_cursor_restricts_catalog() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for index in 0..5 {
        std::fs::write(project.path().join(format!("{index}.rs")),
            "fn authentication_check() {}\n").unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({
        "operation":"locate","query":"authentication check","max_files":2
    })).await;
    assert_eq!(result["catalog_complete"], false);
    assert_eq!(result["ranking_exhaustive"], false);
    assert_eq!(result["truncated_reason"], "max_files");
    assert!(!result["results"].as_array().unwrap().is_empty());

    let page = run(&tool, json!({
        "operation":"locate","query":"authentication check","after_path":"1.rs"
    })).await;
    assert_eq!(page["catalog_complete"], false);
    assert_eq!(page["ranking_exhaustive"], false);
}

#[tokio::test]
async fn locate_refuses_blank_queries_and_never_indexes_hidden_credentials() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("main.rs"), "fn authentication_check() {}\n").unwrap();
    std::fs::write(project.path().join(".env"), "authentication_check_secret = do_not_read").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    assert!(tool.execute(json!({"operation":"locate","query":"  "})).await.is_err());
    let result = run(&tool, json!({"operation":"locate","query":"authentication check"})).await;
    assert!(result.to_string().find(".env").is_none());
    assert_eq!(result["results"][0]["path"], "main.rs");
}


#[tokio::test]
async fn source_pages_follow_global_lexical_order_even_with_directory_prefixes() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("a")).unwrap();
    std::fs::write(project.path().join("a/child.rs"), "fn nested() {}\n").unwrap();
    std::fs::write(project.path().join("a.rs"), "fn sibling() {}\n").unwrap();
    std::fs::write(project.path().join("b.rs"), "fn final_file() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let first = run(&tool, json!({"operation":"scan","max_files":1})).await;
    assert_eq!(first["files"][0]["path"], "a.rs");
    assert_eq!(first["next_after_path"], "a.rs");
    let second = run(&tool, json!({
        "operation":"scan","max_files":1,"after_path":"a.rs"
    })).await;
    assert_eq!(second["files"][0]["path"], "a/child.rs");
    let third = run(&tool, json!({
        "operation":"scan","max_files":1,"after_path":"a/child.rs"
    })).await;
    assert_eq!(third["files"][0]["path"], "b.rs");
    assert_eq!(third["complete"], true);
    assert_eq!(third["next_after_path"], Value::Null);
}

#[tokio::test]
async fn large_partial_graph_remains_valid_json_after_enrichment() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    for index in 0..32 {
        let stem = format!("{}_{index:02}", "a".repeat(120));
        let content = (0..32).map(|other| {
            format!("import './{}_{other:02}';\n", "a".repeat(120))
        }).collect::<String>();
        std::fs::write(project.path().join(format!("{stem}.ts")), content).unwrap();
    }
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = tool.execute(json!({"operation":"graph","max_files":16})).await.unwrap();
    let raw = result.output();
    assert!(raw.len() <= 28_000, "graph exceeds safe JSON response budget: {}", raw.len());
    let graph: Value = serde_json::from_str(raw).expect("complete JSON graph");
    assert_eq!(graph["catalog_complete"], false);
    assert_eq!(graph["edges_complete"], false);
    assert_eq!(graph["impact"], json!([]));
}


#[tokio::test]
async fn verification_can_reopen_the_exact_match_window_from_a_locate_result() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("long.rs"),
        format!("{}authSessionFresh = true;\n", "🙂".repeat(220))).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let located = run(&tool, json!({
        "operation":"locate","query":"auth session fresh"
    })).await;
    let evidence = &located["results"][0]["evidence"][0];
    let expected = located["results"][0]["source_hash"].as_str().unwrap();
    let result = run(&tool, json!({
        "operation":"verify", "path":"long.rs", "expected_sha256":expected,
        "start_line":1, "end_line":1,
        "start_column":evidence["column_start"]
    })).await;
    assert_eq!(result["verified"], true);
    assert_eq!(result["lines"][0]["column_start"], evidence["column_start"]);
    assert_eq!(result["lines"][0]["text"], evidence["text"]);
    assert!(result["lines"][0]["text"].as_str().unwrap().contains("authSessionFresh"));
}


#[tokio::test]
async fn verification_column_cursor_has_strict_bounds_and_defaults_to_first_column() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("chars.rs"), "🙂abc\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let catalog = run(&tool, json!({"operation":"scan"})).await;
    let hash = catalog["files"][0]["source_hash"].as_str().unwrap();
    let default = run(&tool, json!({
        "operation":"verify", "path":"chars.rs", "expected_sha256":hash,
        "start_line":1, "end_line":1
    })).await;
    assert_eq!(default["lines"][0]["text"], "🙂abc");
    assert_eq!(default["lines"][0]["column_start"], 1);
    let middle = run(&tool, json!({
        "operation":"verify", "path":"chars.rs", "expected_sha256":hash,
        "start_column":2, "start_line":1, "end_line":1
    })).await;
    assert_eq!(middle["lines"][0]["text"], "abc");
    for column in [0_u64, 256 * 1024 + 2] {
        assert!(tool.execute(json!({
            "operation":"verify", "path":"chars.rs", "expected_sha256":hash,
            "start_column":column
        })).await.is_err(), "invalid one-based cursor {column}");
    }
}


#[tokio::test]
async fn graph_does_not_treat_unused_literal_query_as_truncated_file_evidence() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let imports = "import './b';\n".repeat(5);
    std::fs::write(project.path().join("a.ts"), imports).unwrap();
    std::fs::write(project.path().join("b.ts"), "export const b = 1;\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let response = run(&tool, json!({"operation":"graph","query":"import"})).await;
    assert_eq!(response["scan_complete"], true,
        "graph operation must not invoke literal scan match truncation");
    assert_eq!(response["catalog_complete"], true);
    assert_eq!(response["edges"][0]["status"], "resolved");
    assert_eq!(response["edges"][0]["to"], "b.ts");
}


#[tokio::test]
async fn scanned_long_line_excerpt_contains_the_search_hit() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("long.rs"), format!("{}needle\n", "x".repeat(240))).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan","query":"needle"})).await;
    let hit = &result["files"][0]["matches"][0];
    assert!(hit["text"].as_str().unwrap().contains("needle"));
    assert!(hit["column_start"].as_u64().unwrap() > 1);
    assert_eq!(hit["excerpt_truncated"], true);
}


#[tokio::test]
async fn scanned_unicode_line_preserves_utf8_boundaries_in_cropped_excerpt() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("unicode.rs"), format!("{}needle🧠\n", "🙂".repeat(200))).unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan","query":"needle🧠"})).await;
    let hit = &result["files"][0]["matches"][0];
    assert!(hit["text"].as_str().unwrap().contains("needle🧠"));
    assert!(hit["text"].as_str().unwrap().chars().count() <= 160);
    assert!(hit["column_start"].as_u64().unwrap() > 1);
}


#[cfg(unix)]
#[tokio::test]
async fn unix_literal_backslashes_do_not_collide_with_separator_paths() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("src/file.rs"), "fn nested() {}\n").unwrap();
    std::fs::write(project.path().join("src\\file.rs"), "fn literal_backslash() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));

    let result = run(&tool, json!({"operation":"scan"})).await;
    assert_eq!(result["files"].as_array().unwrap().len(), 2);
    let files = result["files"].as_array().unwrap();
    assert_ne!(files[0]["path"], files[1]["path"]);
    assert!(files.iter().any(|f| f["path"] == "src/file.rs"));
    assert!(files.iter().any(|f| f["path"] == "src\\file.rs"));

    let digest = files.iter()
        .find(|f| f["path"] == "src\\file.rs").unwrap()["source_hash"]
        .as_str().unwrap();
    let reread = run(&tool, json!({
        "operation": "verify", "path": "src\\file.rs",
        "expected_sha256": digest, "start_line": 1, "end_line": 1
    })).await;
    assert_eq!(reread["lines"][0]["text"], "fn literal_backslash() {}");
}

#[cfg(unix)]
#[tokio::test]
async fn non_utf8_filename_cannot_be_silently_counted_as_complete_scan() {
    use std::os::unix::ffi::OsStrExt;
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let non_utf8 = std::ffi::OsStr::from_bytes(b"source_\xff.rs");
    std::fs::write(project.path().join(non_utf8), "fn opaque() {}\n").unwrap();
    std::fs::write(project.path().join("good.rs"), "fn good() {}\n").unwrap();

    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let result = run(&tool, json!({"operation":"scan"})).await;
    assert_eq!(result["files"].as_array().unwrap().len(), 1);
    assert_eq!(result["files"][0]["path"], "good.rs");
    assert_eq!(result["complete"], false);
    assert_eq!(result["truncated_reason"], "non_utf8_path");
}


#[cfg(unix)]
#[tokio::test]
async fn unix_backslash_filename_must_remain_usable_as_safe_page_cursor() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("src/file.rs"), "fn directory() {}\n").unwrap();
    std::fs::write(project.path().join("src\\file.rs"), "fn literal() {}\n").unwrap();
    std::fs::write(project.path().join("t.rs"), "fn tail() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let page = run(&tool, json!({
        "operation": "scan", "max_files": 1, "after_path": "src/file.rs"
    })).await;
    assert_eq!(page["files"][0]["path"], "src\\file.rs");
    assert_eq!(page["next_after_path"], "src\\file.rs");
    let final_page = run(&tool, json!({
        "operation": "scan", "max_files": 1, "after_path": page["next_after_path"]
    })).await;
    assert_eq!(final_page["files"][0]["path"], "t.rs");
    assert_eq!(final_page["complete"], true);
}


#[tokio::test]
async fn source_evidence_excludes_workspace_auth_material_without_hiding_auth_source_code() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::create_dir(project.path().join(".openhuman")).unwrap();
    std::fs::create_dir(project.path().join("src")).unwrap();
    for secret in [".openhuman/credentials.toml", ".netrc", ".envrc",
                   ".git-credentials", "auth.json", "token.json",
                   "service-account.json"] {
        std::fs::write(project.path().join(secret), "SOURCE_SECRET_DO_NOT_EXPOSE").unwrap();
    }
    std::fs::write(project.path().join("src/auth.ts"),
        "export function auth() { return true; }\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let scan = run(&tool, json!({
        "operation":"scan", "query":"SOURCE_SECRET_DO_NOT_EXPOSE"
    })).await;
    assert_eq!(scan["complete"], true);
    assert_eq!(scan["files"].as_array().unwrap().len(), 1);
    assert_eq!(scan["files"][0]["path"], "src/auth.ts");
    assert!(!scan.to_string().contains("SOURCE_SECRET_DO_NOT_EXPOSE"));

    for secret in [".openhuman/credentials.toml", ".netrc", ".envrc",
                   ".git-credentials", "auth.json", "token.json",
                   "service-account.json"] {
        assert!(tool.execute(json!({"operation":"scan","path":secret})).await.is_err(),
            "sensitive path must be refused: {secret}");
    }
}


#[tokio::test]
async fn malformed_operation_and_path_never_fall_back_to_workspace_wide_scan() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("visible.rs"), "fn visible() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    for malformed in [
        json!([]),
        json!(null),
        json!({"operation": null}),
        json!({"operation": false}),
        json!({"operation": "sacn"}),
        json!({"path": null}),
        json!({"path": 7}),
        json!({"path": ""}),
        json!({"operation": "scan", "path": {"name": "visible.rs"}}),
        json!({"operation": "scan", "query": ["visible"]}),
        json!({"operation": "scan", "query": null}),
    ] {
        assert!(
            tool.execute(malformed.clone()).await.is_err(),
            "malformed request must fail closed, not scan root: {malformed}"
        );
    }
    // A correctly omitted optional operation/path retains documented defaults.
    let valid = run(&tool, json!({})).await;
    assert_eq!(valid["operation"], "scan");
    assert_eq!(valid["files"][0]["path"], "visible.rs");
}

#[tokio::test]
async fn malformed_numeric_and_filter_arguments_do_not_silently_use_defaults() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("visible.rs"), "fn visible() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    for malformed in [
        json!({"max_files": "1"}),
        json!({"max_files": 0}),
        json!({"max_files": 33}),
        json!({"max_entries": -1}),
        json!({"max_edges": 0}),
        json!({"max_results": 6}),
        json!({"start_line": "1"}),
        json!({"start_column": -1}),
        json!({"operation":"graph", "focus_path": 12}),
        json!({"operation":"scan", "after_path": false}),
        json!({"operation":"verify", "path":"visible.rs", "expected_sha256": null}),
    ] {
        assert!(
            tool.execute(malformed.clone()).await.is_err(),
            "invalid argument should be refused before reading: {malformed}"
        );
    }
}


#[tokio::test]
async fn misspelled_tool_fields_do_not_turn_narrow_requests_into_root_scans() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    std::fs::write(project.path().join("visible.rs"), "fn visible() {}\n").unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    for typo in [
        json!({"op": "verify", "path": "visible.rs"}),
        json!({"operation": "scan", "paht": "visible.rs"}),
        json!({"operation": "scan", "query_text": "visible"}),
        json!({"operation": "scan", "limit": 1}),
        json!({"operation": "graph", "focus": "visible.rs"}),
    ] {
        assert!(tool.execute(typo.clone()).await.is_err(),
            "unknown key must not broaden source access: {typo}");
    }
    let schema = tool.parameters_schema();
    assert_eq!(schema["additionalProperties"], false);
}


#[tokio::test]
async fn rejected_unknown_argument_keys_are_not_echoed_into_model_context() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    let sensitive_key = format!("SECRET_ACCESS_TOKEN_{}", "x".repeat(5000));
    let mut args = json!({"operation":"scan"});
    args.as_object_mut().unwrap().insert(sensitive_key.clone(), Value::Null);
    let message = tool.execute(args).await.unwrap_err().to_string();
    assert!(!message.contains("SECRET_ACCESS_TOKEN"));
    assert!(!message.contains(&sensitive_key));
    assert!(message.len() < 256, "error must stay bounded and non-echoing");
}


#[cfg(unix)]
#[test]
fn file_descriptor_identity_detects_atomic_source_replacement() {
    let dir = tempfile::tempdir().unwrap();
    let original = dir.path().join("source.rs");
    let replacement = dir.path().join("replacement.rs");
    std::fs::write(&original, "fn original() {}\n").unwrap();
    std::fs::write(&replacement, "fn substituted() {}\n").unwrap();
    let before = std::fs::metadata(&original).unwrap();
    let same = std::fs::metadata(&original).unwrap();
    assert!(same_file_identity(&before, &same));
    let different = std::fs::metadata(&replacement).unwrap();
    assert!(!same_file_identity(&before, &different));
    std::fs::rename(&replacement, &original).unwrap();
    let after = std::fs::metadata(&original).unwrap();
    assert!(!same_file_identity(&before, &after),
        "atomically replacing a validated file must invalidate its provenance");
}


#[tokio::test]
async fn oversized_string_arguments_fail_before_attempting_any_path_resolution() {
    let home = tempfile::tempdir().unwrap();
    let project = tempfile::tempdir().unwrap();
    let tool = SourceEvidenceTool::new(policy(home.path(), project.path()));
    for (field, request) in [
        ("path", json!({"operation":"scan","path":"x".repeat(4097)})),
        ("query", json!({
            "operation":"scan", "path":"missing-source.rs", "query":"a".repeat(129)
        })),
        ("after_path", json!({
            "operation":"scan", "path":"missing-source.rs", "after_path":"x".repeat(181)
        })),
        ("focus_path", json!({
            "operation":"graph", "path":"missing-source.rs", "focus_path":"x".repeat(181)
        })),
        ("expected_sha256", json!({
            "operation":"verify", "path":"missing-source.rs", "expected_sha256":"f".repeat(65)
        })),
    ] {
        let message = tool.execute(request).await.unwrap_err().to_string();
        assert!(message.contains(field), "expected early {field} validation, got: {message}");
        assert!(!message.contains("No such file"), "filesystem was consulted before validation");
    }
}
