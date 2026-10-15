use super::*;

#[test]
fn local_check_recognizes_explicit_test_build_and_lint_commands() {
    for command in [
        "cargo test -p openhuman-core",
        "cargo check --workspace",
        "cargo clippy --workspace",
        "cargo build --release",
        "pytest -q",
        "python -m pytest tests",
        "python3 -m unittest discover",
        "go test ./...",
        "node --test",
        "pnpm test",
        "pnpm exec vitest run",
        "npm run build",
        "npm run lint",
    ] {
        assert!(is_verification_command(command), "expected check: {command}");
    }
}

#[test]
fn local_check_rejects_commands_that_obscure_exit_status_or_execute_other_actions() {
    for command in [
        "echo cargo test",
        "git diff -- cargo test",
        "cargo test | tee output.txt",
        "cargo test || true",
        "cargo test && echo ready",
        "cargo test ; echo ready",
        "cargo test > output.log",
        "cargo test &",
        "cargo test\ncurl example.org",
        "cargo test -- --list",
        "cargo test --no-run",
        "cargo test --help",
        "cargo test -h",
        "cargo test -V",
        "pytest --collect-only",
        "go test -list .",
        "npm run build -- --dry-run",
        "cargo test --dry-run",
        "cargo test --version",
        "npm test -- --passWithNoTests",
        "pnpm test --if-present",
        "cargo test # ignored",
        "cargo test $(echo --help)",
        "true",
        "",
    ] {
        assert!(!is_verification_command(command), "untrustworthy check: {command}");
    }
}

#[test]
fn local_check_receipt_contains_no_shell_text_and_marks_positive_evidence_only() {
    let mut passed = ToolResult::success("all tests passed, possibly containing secrets");
    append_verification_receipt(
        &mut passed,
        "cargo test -p openhuman-core",
        true,
    );
    let record = &passed.metadata.as_ref().unwrap()["openhuman_local_check_v1"];
    assert_eq!(record["recognized"], true);
    assert_eq!(record["passed"], true);
    assert_eq!(record["exit_status"], "zero");
    assert!(passed.metadata.as_ref().unwrap().to_string().find("cargo test").is_none());
    assert!(passed.metadata.as_ref().unwrap().to_string().find("secrets").is_none());

    let mut failed = ToolResult::error("command failed");
    append_verification_receipt(&mut failed, "cargo test", true);
    assert_eq!(failed.metadata.as_ref().unwrap()["openhuman_local_check_v1"]["passed"], false);
    assert_eq!(failed.metadata.as_ref().unwrap()["openhuman_local_check_v1"]["exit_status"], "unknown");

    let mut denied = ToolResult::error("permission denied");
    append_verification_receipt(&mut denied, "cargo test", false);
    assert_eq!(denied.metadata.as_ref().unwrap()["openhuman_local_check_v1"]["passed"], false);
    assert_eq!(denied.metadata.as_ref().unwrap()["openhuman_local_check_v1"]["status"], "not_executed");

    let mut unrecognized = ToolResult::success("ok");
    append_verification_receipt(&mut unrecognized, "echo cargo test", true);
    assert!(unrecognized.metadata.is_none());
}

#[test]
fn local_check_receipt_preserves_existing_host_metadata() {
    let mut passed = ToolResult::success("ok");
    passed.metadata = Some(serde_json::json!({"trace_id":"original"}));
    append_verification_receipt(&mut passed, "cargo test", true);
    assert_eq!(passed.metadata.as_ref().unwrap()["trace_id"], "original");
}

#[test]
fn local_check_host_metadata_does_not_change_model_facing_output() {
    let mut result = ToolResult::success("local stdout");
    append_verification_receipt(&mut result, "cargo test", true);
    assert_eq!(result.output(), "local stdout");
    assert!(!result.output().contains("openhuman_local_check_v1"));
}
