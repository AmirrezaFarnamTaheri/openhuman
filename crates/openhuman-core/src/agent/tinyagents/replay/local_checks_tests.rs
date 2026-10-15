use super::*;
use serde_json::json;

fn passing_receipt() -> serde_json::Value {
    json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": true, "exit_status": "zero",
        "scope": "local_check_only", "task_verified": false
    }})
}

#[test]
fn journal_local_checks_follow_known_writes_without_claiming_task_verification() {
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "file_write", LocalToolEvent::Started);
    acc.record(2, "file_write", LocalToolEvent::Completed { metadata: None, error: false });
    let result = passing_receipt();
    acc.record(3, "shell", LocalToolEvent::Started);
    acc.record(4, "shell", LocalToolEvent::Completed { metadata: Some(&result), error: false });
    let status = acc.finish("test-run", true);
    assert_eq!(status.status, "local_check_observed");
    assert_eq!(status.last_check_offset, Some(4));
    assert!(status.check_after_last_known_mutation);
    assert_eq!(status.passed_checks, 1);
    assert!(!status.task_verified, "successful process != verified software task");
    assert!(status.journal_complete);
}

#[test]
fn later_unclassified_shell_can_modify_files_so_it_invalidates_freshness() {
    let result = passing_receipt();
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "shell", LocalToolEvent::Completed { metadata: Some(&result), error: false });
    acc.record(2, "shell", LocalToolEvent::Started);
    acc.record(3, "shell", LocalToolEvent::Completed { metadata: None, error: false });
    let status = acc.finish("test-run", true);
    assert_eq!(status.status, "stale_local_check");
    assert_eq!(status.unclassified_shell_calls, 1);
    assert!(!status.check_after_last_known_mutation);
}

#[test]
fn failed_and_forbidden_checks_never_become_passed_evidence() {
    let bad = json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": false, "status": "not_executed",
        "exit_status": "unknown", "scope": "local_check_only"
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(7, "shell", LocalToolEvent::Completed { metadata: Some(&bad), error: true });
    let status = acc.finish("test-run", true);
    assert_eq!(status.passed_checks, 0);
    assert_eq!(status.failed_checks, 1);
    assert_eq!(status.status, "failed_local_check");
}

#[test]
fn unrecognized_or_malformed_host_metadata_cannot_forge_a_check() {
    let forged = json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": true, "exit_status": "zero",
        "scope": "task_verified", "task_verified": true
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "shell", LocalToolEvent::Completed { metadata: Some(&forged), error: false });
    acc.record(2, "file_read", LocalToolEvent::Completed { metadata: Some(&passing_receipt()), error: false });
    let status = acc.finish("test-run", true);
    assert_eq!(status.passed_checks, 0);
    assert_eq!(status.status, "no_local_check");
    assert_eq!(status.unclassified_shell_calls, 1);
}

#[test]
fn truncated_journal_and_started_mutations_do_not_prove_freshness() {
    let result = passing_receipt();
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "shell", LocalToolEvent::Completed { metadata: Some(&result), error: false });
    acc.record(2, "file_write", LocalToolEvent::Started);
    let status = acc.finish("test-run", false);
    assert_eq!(status.status, "partial_journal");
    assert!(!status.check_after_last_known_mutation);
    assert!(!status.task_verified);
}


#[test]
fn a_later_failed_check_supersedes_earlier_success() {
    let passed = passing_receipt();
    let failed = json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": false, "status": "failed_or_unobserved",
        "exit_status": "unknown", "scope": "local_check_only",
        "task_verified": false
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "file_write", LocalToolEvent::Completed { metadata: None, error: false });
    acc.record(2, "shell", LocalToolEvent::Completed { metadata: Some(&passed), error: false });
    acc.record(3, "shell", LocalToolEvent::Completed { metadata: Some(&failed), error: true });
    let result = acc.finish("run", true);
    assert_eq!(result.passed_checks, 1);
    assert_eq!(result.failed_checks, 1);
    assert_eq!(result.status, "failed_local_check");
    assert!(!result.check_after_last_known_mutation);
}

#[test]
fn a_later_success_can_supersede_an_earlier_failed_check() {
    let passed = passing_receipt();
    let failed = json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": false, "status": "failed_or_unobserved",
        "exit_status": "unknown", "scope": "local_check_only",
        "task_verified": false
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "shell", LocalToolEvent::Completed { metadata: Some(&failed), error: true });
    acc.record(2, "shell", LocalToolEvent::Completed { metadata: Some(&passed), error: false });
    let result = acc.finish("run", true);
    assert_eq!(result.failed_checks, 1);
    assert_eq!(result.status, "local_check_observed");
    assert!(result.check_after_last_known_mutation);
    assert!(!result.task_verified);
}
