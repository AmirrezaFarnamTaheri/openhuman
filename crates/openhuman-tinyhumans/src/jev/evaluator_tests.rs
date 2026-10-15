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


#[test]
fn system_one_intent_keeps_both_head_and_tail_when_over_budget() {
    let mut input = request(None);
    input.intent = format!("original-goal:{}:latest-specific-detail", "🙂".repeat(6000));
    let wire = TinyJevEvaluator::build(&input);
    let text = wire.state["request"].as_str().unwrap();
    assert!(text.starts_with("original-goal:"));
    assert!(text.ends_with(":latest-specific-detail"));
    assert!(text.chars().count() <= 4_080);
    assert_eq!(wire.state["request_truncated"], true);
    assert_eq!(wire.state["context_truncated"], true);
}

#[test]
fn system_one_short_intent_and_context_are_not_flagged_partial() {
    let wire = TinyJevEvaluator::build(&request(None));
    assert_eq!(wire.state["request"], "ping alex");
    assert!(wire.state.get("request_truncated").is_none());
    assert!(wire.state.get("context_truncated").is_none());
}


#[test]
fn jev_choice_cannot_silently_collapse_duplicate_option_keys() {
    let mut input = request(None);
    input.options.push(JevOption {
        key: "SLACK_SEND_MESSAGE".into(),
        description: "different tool with same key".into(),
    });
    assert!(TinyJevEvaluator::validate(&input).is_err());
}

#[test]
fn jev_rejects_excessive_option_catalog_or_oversized_descriptions_before_egress() {
    let mut input = request(None);
    input.options = (0..256).map(|i| JevOption {
        key: format!("option_{i}"),
        description: "tool candidate".into(),
    }).collect();
    assert!(TinyJevEvaluator::validate(&input).is_err());
    input = request(None);
    input.options[0].description = "unbounded".repeat(700);
    assert!(TinyJevEvaluator::validate(&input).is_err());
}


#[test]
fn system_one_preserves_valid_family_level_choices_above_shortlist_size() {
    let mut input = request(None);
    input.options = (0..65).map(|i| JevOption {
        key: format!("tool_{i}"),
        description: "normal family candidate".into(),
    }).collect();
    assert!(TinyJevEvaluator::validate(&input).is_ok(),
        "family expansion may legitimately exceed the default top-20 shortlist");
}


#[test]
fn system_one_total_choice_description_budget_is_bounded() {
    let mut input = request(None);
    input.options = (0..100).map(|i| JevOption {
        key: format!("tool_{i}"),
        description: "x".repeat(1200),
    }).collect();
    assert!(TinyJevEvaluator::validate(&input).is_err(),
        "per-option limits alone cannot bound cumulative token/context cost");
}


#[test]
fn system_one_rejects_unbounded_host_instructions_and_invalid_model_names() {
    let mut input = request(Some(&"repeated configuration text ".repeat(400)));
    assert!(TinyJevEvaluator::validate(&input).is_err());
    input = request(None);
    input.model = "\ninvalid-model".into();
    assert!(TinyJevEvaluator::validate(&input).is_err());
    input.model = " ".into();
    assert!(TinyJevEvaluator::validate(&input).is_err());
}

#[test]
fn system_one_always_treats_request_and_option_descriptions_as_data() {
    let input = request(Some("Which skill family should be chosen?"));
    let wire = TinyJevEvaluator::build(&input);
    let Some(Question::Choice(choice)) = wire.questions.get("tool") else { panic!("missing choice") };
    let instructions = choice.instructions.as_str().unwrap();
    assert!(instructions.contains("Which skill family should be chosen?"));
    assert!(instructions.contains("untrusted data"));
    assert!(instructions.contains("not instructions"));
}

#[test]
fn malformed_probability_vectors_are_rejected_before_tool_ranking() {
    let input = request(None);
    let valid = BTreeMap::from([
        ("SLACK_SEND_MESSAGE".to_string(), 0.8_f64),
        ("none".to_string(), 0.2_f64),
    ]);
    assert!(validate_choice_numbers(&input, &valid, 0.85, Some(0.9)).is_ok());
    for probability in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
        let bad = BTreeMap::from([("SLACK_SEND_MESSAGE".into(), probability)]);
        assert!(validate_choice_numbers(&input, &bad, 0.9, None).is_err());
    }
    assert!(validate_choice_numbers(&input, &valid, f64::NAN, None).is_err());
    assert!(validate_choice_numbers(&input, &valid, 0.9, Some(-0.1)).is_err());
    assert!(validate_choice_numbers(
        &input, &BTreeMap::from([("INVENTED_TOOL".into(), 1.0)]), 0.9, None
    ).is_err());
    assert!(validate_choice_numbers(&input, &BTreeMap::new(), 0.9, None).is_err());
}
