use super::*;
use serde_json::json;

fn passing_receipt() -> serde_json::Value {
    json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": true, "status": "passed",
        "exit_status": "zero", "scope": "local_check_only", "task_verified": false, "check_kind": "cargo_test"
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
        "exit_status": "unknown", "scope": "local_check_only",
        "task_verified": false, "check_kind": "cargo_test"
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
fn a_later_success_cannot_certify_a_failed_suite_without_invocation_identity() {
    let passed = passing_receipt();
    let failed = json!({"openhuman_local_check_v1":{
        "recognized": true, "passed": false, "status": "failed_or_unobserved",
        "exit_status": "unknown", "scope": "local_check_only",
        "task_verified": false, "check_kind": "cargo_test"
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "shell", LocalToolEvent::Completed { metadata: Some(&failed), error: true });
    acc.record(2, "shell", LocalToolEvent::Completed { metadata: Some(&passed), error: false });
    let result = acc.finish("run", true);
    assert_eq!(result.failed_checks, 1);
    assert_eq!(result.status, "failed_local_check");
    assert!(!result.check_after_last_known_mutation);
    assert!(!result.task_verified);
}


#[test]
fn contradictory_or_incomplete_local_check_metadata_cannot_claim_success() {
    let invalid = [
        json!({"openhuman_local_check_v1":{
            "recognized": true, "passed": true, "status": "not_executed",
            "exit_status": "zero", "scope": "local_check_only",
            "task_verified": false
        }}),
        json!({"openhuman_local_check_v1":{
            "recognized": true, "passed": true,
            "exit_status": "zero", "scope": "local_check_only",
            "task_verified": false
        }}),
        json!({"openhuman_local_check_v1":{
            "recognized": true, "passed": true, "status": "passed",
            "exit_status": "zero", "scope": "local_check_only"
        }}),
        json!({"openhuman_local_check_v1":{
            "recognized": true, "passed": false, "status": "passed",
            "exit_status": "unknown", "scope": "local_check_only",
            "task_verified": false
        }}),
    ];
    for fact in invalid {
        let mut acc = LocalCheckAccumulator::default();
        acc.record(1, "shell", LocalToolEvent::Completed {
            metadata: Some(&fact), error: false,
        });
        let report = acc.finish("run", true);
        assert_eq!(report.passed_checks, 0, "invalid receipt: {fact}");
        assert_eq!(report.status, "no_local_check");
        assert_eq!(report.unclassified_shell_calls, 1);
    }
}


#[test]
fn unrelated_formatter_success_does_not_hide_a_failed_test_suite() {
    let failed = json!({"openhuman_local_check_v1":{
        "recognized":true,"passed":false,"status":"failed_or_unobserved",
        "exit_status":"unknown","scope":"local_check_only",
        "task_verified":false,"check_kind":"cargo_test"
    }});
    let formatter = json!({"openhuman_local_check_v1":{
        "recognized":true,"passed":true,"status":"passed",
        "exit_status":"zero","scope":"local_check_only",
        "task_verified":false,"check_kind":"cargo_fmt"
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1, "shell", LocalToolEvent::Completed {metadata:Some(&failed),error:true});
    acc.record(2, "shell", LocalToolEvent::Completed {metadata:Some(&formatter),error:false});
    let report = acc.finish("a",true);
    assert_eq!(report.status,"failed_local_check");
    assert!(!report.check_after_last_known_mutation);
    assert_eq!(report.unresolved_check_kinds,vec!["cargo_test"]);
}

#[test]
fn same_check_category_cannot_prove_that_the_failed_invocation_was_rerun() {
    let failed = json!({"openhuman_local_check_v1":{
        "recognized":true,"passed":false,"status":"failed_or_unobserved",
        "exit_status":"unknown","scope":"local_check_only",
        "task_verified":false,"check_kind":"cargo_test"
    }});
    let passed = passing_receipt();
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1,"shell",LocalToolEvent::Completed{metadata:Some(&failed),error:true});
    acc.record(2,"shell",LocalToolEvent::Completed{metadata:Some(&passed),error:false});
    let report = acc.finish("a",true);
    assert_eq!(report.status,"failed_local_check");
    assert_eq!(report.unresolved_check_kinds,vec!["cargo_test"]);
    assert!(!report.check_after_last_known_mutation);
}

#[test]
fn unknown_plugin_tool_after_check_invalidates_local_freshness() {
    let passed = passing_receipt();
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1,"shell",LocalToolEvent::Completed{metadata:Some(&passed),error:false});
    acc.record(2,"unknown_plugin_write_file",LocalToolEvent::Started);
    acc.record(3,"unknown_plugin_write_file",LocalToolEvent::Completed{metadata:None,error:false});
    let report = acc.finish("a",true);
    assert_eq!(report.status,"stale_local_check");
    assert!(!report.check_after_last_known_mutation);
    assert_eq!(report.last_possible_mutation_offset,Some(3));
}

#[test]
fn allowlisted_read_only_tools_preserve_check_freshness() {
    let passed = passing_receipt();
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1,"shell",LocalToolEvent::Completed{metadata:Some(&passed),error:false});
    for name in ["file_read","grep","glob","source_evidence"] {
        acc.record(2,name,LocalToolEvent::Started);
        acc.record(3,name,LocalToolEvent::Completed{metadata:None,error:false});
    }
    let report = acc.finish("a",true);
    assert_eq!(report.status,"local_check_observed");
    assert!(report.check_after_last_known_mutation);
}

#[test]
fn legacy_unattributed_failure_cannot_be_erased_by_unrelated_success() {
    let failure = json!({"openhuman_local_check_v1":{
        "recognized":true,"passed":false,"status":"failed_or_unobserved",
        "exit_status":"unknown","scope":"local_check_only","task_verified":false
    }});
    let passed = passing_receipt();
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1,"shell",LocalToolEvent::Completed{metadata:Some(&failure),error:true});
    acc.record(2,"shell",LocalToolEvent::Completed{metadata:Some(&passed),error:false});
    let report = acc.finish("a",true);
    assert_eq!(report.status,"failed_local_check");
    assert_eq!(report.unattributed_failures,1);
}

#[test]
fn unsupported_check_kind_cannot_forge_a_successful_receipt() {
    let forged = json!({"openhuman_local_check_v1":{
        "recognized":true,"passed":true,"status":"passed","exit_status":"zero",
        "scope":"local_check_only","task_verified":false,
        "check_kind":"third_party_authoritative_ci"
    }});
    let mut acc = LocalCheckAccumulator::default();
    acc.record(1,"shell",LocalToolEvent::Completed{metadata:Some(&forged),error:false});
    let report = acc.finish("a",true);
    assert_eq!(report.status,"no_local_check");
    assert_eq!(report.passed_checks,0);
}


#[test]
fn independent_clean_journal_can_report_a_new_successful_check() {
    let passed = passing_receipt();
    let mut clean = LocalCheckAccumulator::default();
    clean.record(1, "file_write", LocalToolEvent::Completed {metadata:None,error:false});
    clean.record(2, "shell", LocalToolEvent::Completed {metadata:Some(&passed),error:false});
    let result = clean.finish("new-run",true);
    assert_eq!(result.status,"local_check_observed");
    assert!(result.unresolved_check_kinds.is_empty());
    assert_eq!(result.unattributed_failures,0);
}
