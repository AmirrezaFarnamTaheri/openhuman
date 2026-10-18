use super::*;

use std::sync::Mutex;

use async_trait::async_trait;
use tinyagents_harness::limits::RunLimits;
use tinyagents_harness::middleware::{Middleware, ToolInvocationIdentity};
use tinyagents_harness::runtime::{AgentHarness, RunPolicy};
use tinyagents_harness::testkit::ScriptedModel;
use tinyagents_harness::tool::packs::{PackRegistryHandle, UseSkillTool};
use tinyagents_harness::tool::{CanonicalSharedToolAdapter, ToolRulePolicy};
use tinyinference_llm::message::Message;
use tinyinference_llm::model::ModelResponse;
use tinyinference_llm::tool::ToolCall;
use tinytools::PermissionLevel;

#[derive(Debug, PartialEq)]
struct Observation {
    call_id: String,
    outer: String,
    target: String,
    args: Value,
    permission: PermissionLevel,
    external_effect: bool,
}

#[derive(Default)]
struct Capture(Mutex<Vec<Observation>>);

#[async_trait]
impl Middleware<()> for Capture {
    fn name(&self) -> &str {
        "capture_structural_execution"
    }

    fn observe_tool_execution(
        &self,
        _ctx: &RunContext<()>,
        _state: &(),
        invocation: &ToolInvocationIdentity,
        target: &dyn Tool,
        arguments: &Value,
    ) {
        self.0.lock().unwrap().push(Observation {
            call_id: invocation.call_id().as_str().to_owned(),
            outer: invocation.tool_name().to_owned(),
            target: target.name().to_owned(),
            args: arguments.clone(),
            permission: target.permission_level_with_args(arguments),
            external_effect: target.external_effect_with_args(arguments),
        });
    }
}

async fn run(
    fixture: &Fixture,
    call: ToolCall,
    denied_target: Option<&str>,
) -> (tinyagents_harness::AgentRun, Arc<Capture>) {
    // Resolve the actual host factory tools through the canonical shared
    // adapter. Packing those adapters exercises the same metadata boundary as
    // the host's private UseSkillDispatch; that selector has sibling unit tests.
    let targets: Arc<Vec<Box<dyn Tool>>> = Arc::new(
        READ_TOOLS
            .into_iter()
            .chain(WRITE_TOOLS)
            .map(|name| {
                Box::new(
                    CanonicalSharedToolAdapter::for_name(vec![fixture.tools.clone()], name)
                        .unwrap(),
                ) as Box<dyn Tool>
            })
            .collect(),
    );
    let handle = PackRegistryHandle::default();
    handle.bind(Arc::downgrade(&targets));
    let mut harness: AgentHarness<()> = AgentHarness::new();
    for name in READ_TOOLS.into_iter().chain(WRITE_TOOLS) {
        harness.register_tool(Arc::new(
            CanonicalSharedToolAdapter::for_name(vec![targets.clone()], name).unwrap(),
        ));
    }
    harness.register_tool(Arc::new(UseSkillTool::new(handle, toolpacks::CATALOG)));
    let mut request = ModelResponse::assistant("");
    request.message.tool_calls = vec![call];
    request.finish_reason = Some("tool_calls".to_owned());
    harness.register_model(
        "mock",
        Arc::new(ScriptedModel::new(vec![
            request,
            ModelResponse::assistant("draft"),
            ModelResponse::assistant("checked"),
        ])),
    );
    let mut policy = RunPolicy {
        limits: RunLimits::default().with_max_model_calls(10),
        ..RunPolicy::default()
    };
    if let Some(name) = denied_target {
        policy.tool_rules = ToolRulePolicy::new(
            serde_json::from_value(json!({
                "rules": [{"effect": "deny", "match": {"name": name}}]
            }))
            .unwrap(),
        );
    }
    harness.with_policy(policy);
    let capture = Arc::new(Capture::default());
    harness.push_middleware(capture.clone());
    finish_policy::install(&mut harness, false, Some("orchestrator"), &None, None);
    let result = harness
        .invoke_in_context(
            &(),
            fixture.run_context(),
            vec![Message::user("complete the requested code change")],
        )
        .await
        .unwrap();
    (result, capture)
}

#[tokio::test]
async fn actual_structural_writes_trigger_the_installed_finish_check_but_previews_do_not() {
    for route in [Route::Direct, Route::Packed] {
        for name in WRITE_TOOLS {
            for preview in [true, false] {
                let fixture = Fixture::new(AutonomyLevel::Full);
                let args = fixture.mutation_args(name, preview).await;
                let (outer, outer_args) = route.call(name, args.clone());
                let call = ToolCall::new("structural-call", &outer, outer_args);
                let (result, capture) = run(&fixture, call, None).await;
                assert_eq!(
                    result.text().as_deref(),
                    Some(if preview { "draft" } else { "checked" })
                );
                assert_eq!(
                    result
                        .messages
                        .iter()
                        .any(|message| message.text().contains(finish_policy::CHECK_MARKER)),
                    !preview
                );
                let observations = capture.0.lock().unwrap();
                assert_eq!(observations.len(), 1);
                assert_eq!(observations[0].call_id, "structural-call");
                assert_eq!(observations[0].outer, outer);
                assert_eq!(observations[0].target, name);
                assert_eq!(observations[0].args, args);
                assert_eq!(
                    observations[0].permission,
                    if preview {
                        PermissionLevel::ReadOnly
                    } else {
                        PermissionLevel::Write
                    }
                );
                assert!(
                    !observations[0].external_effect,
                    "local Full-autonomy writes are classified by permission"
                );
                assert!(result.messages.iter().any(|message| matches!(message,
                    Message::Assistant(assistant) if assistant.tool_calls.iter().any(|call|
                        call.id == "structural-call" && call.name == outer)
                )));
                if preview {
                    fixture.assert_original_files();
                } else if name == "ast_patch" {
                    assert!(std::fs::read_to_string(fixture.project.join("source.rs"))
                        .unwrap()
                        .contains('2'));
                } else {
                    assert!(std::fs::read_to_string(fixture.project.join("new.rs"))
                        .unwrap()
                        .contains("fn created"));
                }
            }
        }
    }
}

#[tokio::test]
async fn an_admission_denial_never_creates_execution_evidence_or_writes() {
    for route in [Route::Direct, Route::Packed] {
        for name in WRITE_TOOLS {
            let fixture = Fixture::new(AutonomyLevel::Full);
            let args = fixture.mutation_args(name, false).await;
            let (outer, args) = route.call(name, args);
            let (result, capture) = run(
                &fixture,
                ToolCall::new("denied-call", outer, args),
                Some(name),
            )
            .await;
            assert_eq!(result.text().as_deref(), Some("draft"));
            assert!(capture.0.lock().unwrap().is_empty());
            assert!(!result
                .messages
                .iter()
                .any(|message| message.text().contains(finish_policy::CHECK_MARKER)));
            fixture.assert_original_files();
        }
    }
}

#[tokio::test]
async fn an_entered_write_refused_by_filesystem_policy_remains_an_attempt_not_success() {
    let fixture = Fixture::new(AutonomyLevel::ReadOnly);
    let args = fixture.mutation_args("ast_patch", false).await;
    let (outer, args) = Route::Packed.call("ast_patch", args);
    let (result, capture) = run(&fixture, ToolCall::new("refused-write", outer, args), None).await;
    assert_eq!(result.text().as_deref(), Some("checked"));
    assert_eq!(
        capture.0.lock().unwrap()[0].permission,
        PermissionLevel::Write
    );
    assert!(result
        .messages
        .iter()
        .any(|message| matches!(message, Message::Tool(_))
            && message.text().contains("policy-blocked")));
    fixture.assert_original_files();
}

#[tokio::test]
async fn disclosure_and_missing_packed_targets_cannot_spoof_structural_activity() {
    for args in [
        json!({"skill": "coding", "args": {"tools_executed": ["ast_patch"], "tools_with_side_effects": ["ast_patch"]}}),
        json!({"skill": "coding", "tool": "missing", "args": {"tool": "ast_patch"}}),
        json!({"skill": "missing", "tool": "ast_patch", "args": {"dry_run": false}}),
    ] {
        let fixture = Fixture::new(AutonomyLevel::Full);
        let (result, capture) = run(
            &fixture,
            ToolCall::new("pack-answer", "use_skill", args),
            None,
        )
        .await;
        assert_eq!(result.text().as_deref(), Some("draft"));
        assert!(capture
            .0
            .lock()
            .unwrap()
            .iter()
            .all(|observation| observation.target == "use_skill"));
        fixture.assert_original_files();
    }
}
