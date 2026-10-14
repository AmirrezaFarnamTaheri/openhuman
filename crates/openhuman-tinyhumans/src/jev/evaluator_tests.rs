use tinytools_jev::JevOption;

use super::*;

fn request(instructions: Option<&str>) -> JevRequest {
    JevRequest {
        intent: "ping alex".into(),
        recent_turns: vec!["earlier turn".into()],
        options: vec![
            JevOption {
                key: "SLACK_SEND_MESSAGE".into(),
                description: "send a message (from slack)".into(),
            },
            JevOption {
                key: "none".into(),
                description: "No listed tool accomplishes the request.".into(),
            },
        ],
        model: "jev-latest".into(),
        instructions: instructions.map(str::to_owned),
    }
}

#[test]
fn builds_one_choice_and_one_noul_from_the_request() {
    let wire = TinyJevEvaluator::build(&request(None));
    assert_eq!(wire.model, "jev-latest");
    assert_eq!(wire.state["request"], "ping alex");
    assert_eq!(wire.state["recent_user_turns"][0], "earlier turn");
    assert_eq!(wire.questions.len(), 2);
    match wire.questions.get("tool") {
        Some(Question::Choice(choice)) => {
            assert_eq!(choice.criteria.len(), 2);
            assert!(choice
                .instructions
                .as_str()
                .unwrap()
                .starts_with("Which tool"));
        }
        other => panic!("expected a choice, got {other:?}"),
    }
    assert!(matches!(
        wire.questions.get("needs_tool"),
        Some(Question::Noul(_))
    ));
}

#[test]
fn request_instructions_replace_the_default_wording() {
    let wire = TinyJevEvaluator::build(&request(Some("Which group of tools applies?")));
    match wire.questions.get("tool") {
        Some(Question::Choice(choice)) => {
            assert_eq!(choice.instructions, json!("Which group of tools applies?"));
        }
        other => panic!("expected a choice, got {other:?}"),
    }
}

#[test]
fn failures_map_without_leaking_the_key() {
    let failure = EvaluationFailure {
        error: JevError::Authentication,
        attempts: 2,
        latency: Duration::from_millis(5),
    };
    let text = map_failure(failure).to_string();
    assert!(text.contains("authentication failed"), "{text}");
    assert!(text.contains("2 attempt(s)"), "{text}");
    assert!(matches!(
        map_failure(EvaluationFailure {
            error: JevError::Timeout,
            attempts: 1,
            latency: Duration::ZERO,
        }),
        RankError::Timeout
    ));
}


#[test]
fn decision_context_is_bounded_and_keeps_latest_turns_in_order() {
    let mut input = request(None);
    input.recent_turns = (0..20).map(|i| format!("user-turn-{i}:{}", "x".repeat(900))).collect();
    let output = TinyJevEvaluator::build(&input);
    let turns = output.state["recent_user_turns"].as_array().unwrap();
    assert_eq!(turns.len(), 6, "external System-One receives only six recent turns");
    assert!(turns[0].as_str().unwrap().starts_with("user-turn-14:"));
    assert!(turns[5].as_str().unwrap().starts_with("user-turn-19:"));
    assert!(turns.iter().all(|t| t.as_str().unwrap().chars().count() <= 500));
    assert_eq!(output.state["context_truncated"], true);
    assert_eq!(output.state["recent_turns_dropped"], 14);
}

#[test]
fn decision_context_ignores_empty_turns_and_preserves_unicode_boundaries() {
    let mut input = request(None);
    input.recent_turns = vec![
        " ".into(),
        format!("{}{}", "🙂".repeat(650), "tail"),
        "newest request".into()
    ];
    let output = TinyJevEvaluator::build(&input);
    let turns = output.state["recent_user_turns"].as_array().unwrap();
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].as_str().unwrap().chars().count(), 500);
    assert_eq!(turns[1], "newest request");
    assert_eq!(output.state["context_truncated"], true);
}
