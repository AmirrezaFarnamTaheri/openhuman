use super::*;
use serde_json::{json, Value};

fn packet() -> Value {
    json!({"type":"coding_handoff", "version":1, "objective":"Check the parser change"})
}

fn context(packet: &Value) -> String {
    format!("{MARKER}\n{packet}")
}

fn rendered(packet: &Value) -> String {
    prepare(context(packet)).unwrap()
}

fn decoded(rendered: &str) -> Value {
    serde_json::from_str(rendered.strip_prefix(REPORT_NOTICE).unwrap()).unwrap()
}

#[test]
fn preserves_legacy_context_bytes_without_interpreting_json_or_embedded_markers() {
    assert_eq!(prepare_context(None).unwrap(), None);
    for source in [
        "".into(),
        "  prior result: 42\n".into(),
        "{\"type\":\"coding_handoff\",\"version\":99}".into(),
        "notes before [coding_handoff]\nnot JSON".into(),
        "legacy context".repeat(4000),
    ] {
        assert_eq!(prepare(source.clone()).unwrap(), source);
    }
}

#[test]
fn renders_exact_reported_commands_and_revisions_as_untrusted_data() {
    let revision = "a".repeat(64);
    let previous = "b".repeat(64);
    let command = "cargo test --lib -- 'quoted name'\nprintf '%s' '$HOME'";
    let source = json!({
        "type":"coding_handoff", "version":1, "objective":"Review the selected function",
        "source_refs":[{"path":"src/分析.rs", "revision":revision, "selected_symbols":["Parser::parse"]}],
        "changed_files":[{"path":"src/分析.rs", "previous_revision":previous, "revision":revision}],
        "intended_edits":["Preserve the existing parser behavior"],
        "checks":[{"command":command,"cwd":"C:\\work tree\\project","status":"passed","exit_code":0,
                   "source_revisions":[{"path":"src/分析.rs","revision":revision}],"summary":"3 tests passed"}],
        "risks":["Cross-file typing has not been checked"], "next_action":"Run the compiler"
    });
    let text = rendered(&source);
    assert!(text.contains("Verification: unverified"));
    assert!(text.contains("caller-reported"));
    assert!(text.contains("grants no access and executes nothing"));
    let report = decoded(&text);
    assert_eq!(report["checks"][0]["command"], command);
    assert_eq!(report["checks"][0]["cwd"], "C:\\work tree\\project");
    assert_eq!(
        report["source_refs"][0]["selected_symbols"][0],
        "Parser::parse"
    );
    assert_eq!(report["changed_files"][0]["previous_revision"], previous);
    assert_eq!(
        report["checks"][0]["source_revisions"][0]["revision"],
        revision
    );
    assert_eq!(
        prepare(text.clone()).unwrap(),
        text,
        "repeated admission must not wrap twice"
    );
}

#[test]
fn missing_skipped_and_cancelled_observations_remain_unverified() {
    let mut source = packet();
    source["checks"] = json!([
        {"command":"cargo test", "cwd":"."},
        {"command":"cargo clippy", "cwd":".", "status":"skipped"},
        {"command":"pnpm test", "cwd":"/project", "status":"cancelled", "exit_code":130}
    ]);
    let text = rendered(&source);
    let report = decoded(&text);
    assert_eq!(report["checks"][0]["status"], "unverified");
    assert_eq!(report["checks"][0]["exit_code"], Value::Null);
    assert_eq!(report["checks"][1]["status"], "skipped");
    assert_eq!(report["checks"][2]["status"], "cancelled");
    assert!(text.contains("missing, skipped and cancelled checks remain unverified"));
    assert!(rendered(&packet()).contains("Verification: unverified"));
}

#[test]
fn reported_success_and_failure_require_consistent_observable_exit_codes() {
    for (status, code, accepted) in [
        ("passed", json!(0), true),
        ("passed", Value::Null, false),
        ("passed", json!(1), false),
        ("failed", json!(2), true),
        ("failed", json!(-1), true),
        ("failed", json!(0), false),
        ("failed", Value::Null, false),
    ] {
        let mut source = packet();
        source["checks"] =
            json!([{"command":"cargo test", "cwd":".", "status":status, "exit_code":code}]);
        assert_eq!(
            prepare(context(&source)).is_ok(),
            accepted,
            "{status}: {code}"
        );
    }
}

#[test]
fn explicit_marker_rejects_malformed_unknown_or_wrongly_typed_contracts() {
    for json in [
        "",
        "{",
        "null",
        "[]",
        r#"{"type":"coding_handoff","version":1,"objective":"x","objective":"duplicate"}"#,
        r#"{"type":"coding_handoff","version":1,"objective":"x","checks":[{"command":"a","command":"b","cwd":"."}]}"#,
        r#"{"type":"other","version":1,"objective":"x"}"#,
        r#"{"type":"coding_handoff","version":"1","objective":"x"}"#,
        r#"{"type":"coding_handoff","version":1,"objective":"x","secret payload":"never echo this"}"#,
    ] {
        let error = prepare(format!("{MARKER}\n{json}")).unwrap_err();
        assert!(error.starts_with("coding_handoff:"));
        assert!(!error.contains("never echo this"));
    }
    let mut source = packet();
    source["version"] = json!(2);
    assert!(prepare(context(&source))
        .unwrap_err()
        .contains("unsupported version"));
    source["version"] = json!(1);
    source["checks"] = json!([{"command":"x", "cwd":".", "status":"trusted"}]);
    assert!(prepare(context(&source)).is_err());
    source["checks"] = json!([{"command":"x", "cwd":".", "verified":true}]);
    assert!(prepare(context(&source)).is_err());
}

#[test]
fn rejects_positional_records_and_object_encoded_unit_enums() {
    let revision = "a".repeat(64);
    let mut invalid = vec![json!(["coding_handoff", 1, "x", [], [], [], [], [], null])];
    let mut source = packet();
    source["type"] = json!({"coding_handoff": null});
    invalid.push(source);
    for (field, record) in [
        ("source_refs", json!(["src/lib.rs", revision, []])),
        ("changed_files", json!(["src/lib.rs", null, revision])),
        ("checks", json!(["cargo test", ".", "passed", 0, [], null])),
    ] {
        let mut source = packet();
        source[field] = json!([record]);
        invalid.push(source);
    }
    let mut source = packet();
    source["checks"] = json!([{
        "command": "cargo test", "cwd": ".", "status": {"passed": null}, "exit_code": 0
    }]);
    invalid.push(source);
    let mut source = packet();
    source["checks"] = json!([{
        "command": "cargo test", "cwd": ".",
        "source_revisions": [["src/lib.rs", revision]]
    }]);
    invalid.push(source);
    let accepted = invalid
        .into_iter()
        .filter(|source| prepare(context(source)).is_ok())
        .collect::<Vec<_>>();
    assert!(accepted.is_empty(), "accepted {accepted:?}");
}

#[test]
fn bounds_input_and_rendered_context_in_utf8_bytes() {
    assert!(
        prepare(format!("{MARKER}{}", " ".repeat(MAX_CONTEXT_BYTES)))
            .unwrap_err()
            .contains("packet exceeds")
    );
    let mut source = packet();
    source["objective"] = json!("文".repeat(800));
    assert!(prepare(context(&source))
        .unwrap_err()
        .contains("UTF-8 bytes"));

    let mut risks = vec!["x".repeat(2048); 15];
    risks.push("x".repeat(1000));
    source["objective"] = json!("x");
    source["risks"] = json!(risks);
    let remaining = MAX_CONTEXT_BYTES - 1 - context(&source).len();
    risks[15].push_str(&"x".repeat(remaining));
    source["risks"] = json!(risks);
    assert!(context(&source).len() < MAX_CONTEXT_BYTES);
    assert!(prepare(context(&source))
        .unwrap_err()
        .contains("rendered context exceeds"));
}

#[test]
fn source_references_are_bounded_relative_paths_and_content_revisions() {
    for path in [
        "",
        ".",
        "..",
        "../lib.rs",
        "/lib.rs",
        "src//lib.rs",
        "src/./lib.rs",
        "C:/lib.rs",
        "src\\lib.rs",
        "https://example.com/a",
        "src/\0lib.rs",
    ] {
        let mut source = packet();
        source["source_refs"] = json!([{"path":path,"revision":"a".repeat(64)}]);
        assert!(prepare(context(&source)).is_err(), "accepted {path:?}");
    }
    for revision in ["", "main", "a123", &"A".repeat(64), &"g".repeat(64)] {
        let mut source = packet();
        source["source_refs"] = json!([{"path":"src/lib.rs","revision":revision}]);
        assert!(prepare(context(&source)).is_err());
    }
    let mut source = packet();
    source["source_refs"] = json!([{"path":"does-not-exist/lib.rs","revision":"a".repeat(64)}]);
    assert!(
        prepare(context(&source)).is_ok(),
        "references must not be opened while decoding"
    );
}

#[test]
fn rejects_unbounded_records_and_ambiguous_duplicate_paths() {
    for field in ["source_refs", "changed_files"] {
        let mut source = packet();
        source[field] = json!((0..33)
            .map(|n| json!({"path":format!("{n}.rs"),"revision":"a".repeat(64)}))
            .collect::<Vec<_>>());
        assert!(prepare(context(&source))
            .unwrap_err()
            .contains("exceeds 32"));
        source[field] = json!([
            {"path":"src/lib.rs","revision":"a".repeat(64)},
            {"path":"src/lib.rs","revision":"b".repeat(64)}
        ]);
        assert!(prepare(context(&source))
            .unwrap_err()
            .contains("duplicate source path"));
    }
    let mut source = packet();
    source["checks"] = json!((0..17)
        .map(|_| json!({"command":"test","cwd":"."}))
        .collect::<Vec<_>>());
    assert!(prepare(context(&source))
        .unwrap_err()
        .contains("exceeds 16"));
    source["checks"] = json!([]);
    source["source_refs"] = json!([{"path":"src/lib.rs","revision":"a".repeat(64),"selected_symbols":vec!["function";17]}]);
    assert!(prepare(context(&source))
        .unwrap_err()
        .contains("selected_symbols exceeds"));
}

#[test]
fn creation_and_deletion_retain_the_known_revision_and_require_one() {
    let mut source = packet();
    source["changed_files"] = json!([
        {"path":"new.rs","revision":"a".repeat(64)},
        {"path":"old.rs","previous_revision":"b".repeat(64)}
    ]);
    let report = decoded(&rendered(&source));
    assert_eq!(report["changed_files"][0]["previous_revision"], Value::Null);
    assert_eq!(report["changed_files"][1]["revision"], Value::Null);
    source["changed_files"] = json!([{"path":"unknown.rs"}]);
    assert!(prepare(context(&source))
        .unwrap_err()
        .contains("previous or current revision"));
}

#[test]
fn rejects_blank_or_unbounded_prose_commands_paths_and_symbols() {
    for field in ["objective", "next_action"] {
        for value in [" ".to_string(), "x".repeat(2049), "bad\0text".to_string()] {
            let mut source = packet();
            source[field] = json!(value);
            assert!(prepare(context(&source)).is_err());
        }
    }
    for field in ["intended_edits", "risks"] {
        let mut source = packet();
        source[field] = json!(vec!["item"; 33]);
        assert!(prepare(context(&source)).is_err());
        source[field] = json!([""]);
        assert!(prepare(context(&source)).is_err());
    }
    let mut source = packet();
    source["checks"] = json!([{"command":"test","cwd":"bad\npath"}]);
    assert!(prepare(context(&source)).is_err());
    source["checks"] = json!([{"command":"test","cwd":".","summary":"x".repeat(2049)}]);
    assert!(prepare(context(&source)).is_err());
    source["checks"] = json!([]);
    source["source_refs"] = json!([{"path":"x".repeat(1025),"revision":"a".repeat(64)}]);
    assert!(prepare(context(&source)).is_err());
    source["source_refs"] =
        json!([{"path":"a.rs","revision":"a".repeat(64),"selected_symbols":["x".repeat(257)]}]);
    assert!(prepare(context(&source)).is_err());
}
