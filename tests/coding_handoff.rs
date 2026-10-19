//! The bounded context contract through the production worker lifecycle and
//! public tool entry points. No live model or referenced command is executed.

// Compile the helper's actual source and adjacent unit tests in this small
// integration target as well as the core's monolithic lib-test target.
#[path = "../crates/openhuman-core/src/agent/orchestration/coding_handoff/mod.rs"]
mod coding_handoff;

use async_trait::async_trait;
use openhuman_core::agent::harness::ParentExecutionContext;
use openhuman_core::agent::orchestration::tools::{
    SpawnAsyncSubagentTool, SpawnParallelAgentsTool, SpawnSubagentTool,
};
use openhuman_core::agent::subagent_host::{run_subagent, SubagentRunError, SubagentRunOptions};
use openhuman_core::agent_registry::host_definition::{AgentDefinition, AgentDefinitionRegistry};
use openhuman_core::config::AgentConfig;
use openhuman_core::security::SecurityPolicy;
use openhuman_core::tools::DelegateTool;
use parking_lot::Mutex;
use serde_json::json;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tinyagents_definition::{ModelSpec, PromptSource, ToolScope};
use tinyinference_llm::message::{Message, MessageDelta};
use tinyinference_llm::model::{
    ChatModel, ModelProfile, ModelRequest, ModelResponse, ModelStream, ModelStreamItem,
};
use tinytools::Tool;
use tinytools_agent::dialect::ToolCallFormat;

#[derive(Default)]
struct RecordingModel {
    requests: Mutex<Vec<ModelRequest>>,
}

impl RecordingModel {
    fn response(&self, request: ModelRequest) -> ModelResponse {
        self.requests.lock().push(request);
        ModelResponse::assistant("Reviewed the supplied context")
    }
}

#[async_trait]
impl ChatModel<()> for RecordingModel {
    fn profile(&self) -> Option<&ModelProfile> {
        static PROFILE: std::sync::LazyLock<ModelProfile> =
            std::sync::LazyLock::new(|| ModelProfile {
                tool_calling: true,
                parallel_tool_calls: true,
                streaming: true,
                ..Default::default()
            });
        Some(&PROFILE)
    }

    async fn invoke(
        &self,
        _: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelResponse> {
        Ok(self.response(request))
    }

    async fn stream(
        &self,
        _: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelStream> {
        let response = self.response(request);
        Ok(ModelStream::new(Box::pin(futures::stream::iter([
            ModelStreamItem::Started,
            ModelStreamItem::MessageDelta(MessageDelta::text(response.text())),
            ModelStreamItem::Completed(response),
        ]))))
    }
}

fn parent(model: Arc<RecordingModel>, workspace: &Path) -> ParentExecutionContext {
    ParentExecutionContext {
        workspace_descriptor: None,
        agent_definition_id: "orchestrator".into(),
        allowed_subagent_ids: ["critic".to_string()].into_iter().collect(),
        turn_model_source: openhuman_core::agent::tinyagents::TurnModelSource::from_model(model),
        all_tools: Arc::new(vec![]),
        all_tool_specs: Arc::new(vec![]),
        visible_tool_specs: Arc::new(vec![]),
        visible_tool_names: Default::default(),
        subagent_tool_ceiling_names: Default::default(),
        model_name: "test-model".into(),
        temperature: 0.5,
        workspace_dir: workspace.to_path_buf(),
        agent_config: AgentConfig::default(),
        workflows: Arc::new(vec![]),
        memory_context: Arc::new(None),
        session_id: "coding-handoff-integration".into(),
        channel: "test".into(),
        connected_integrations: vec![],
        tool_call_format: ToolCallFormat::PFormat,
        session_key: "0_coding_handoff".into(),
        session_parent_prefix: None,
        on_progress: None,
        run_queue: None,
        host_env: Default::default(),
    }
}

fn definition() -> AgentDefinition {
    AgentDefinitionRegistry::init_global_builtins().expect("initialize built-in definitions");
    let mut definition = AgentDefinitionRegistry::global()
        .expect("registry")
        .get("critic")
        .expect("critic")
        .clone();
    definition.model = ModelSpec::Inherit;
    definition.system_prompt = PromptSource::Inline("Review the supplied report".into());
    definition.tools = ToolScope::Named(vec![]);
    definition.max_iterations = 2;
    definition.extensions.omit_identity = true;
    definition.extensions.omit_memory_context = true;
    definition.extensions.omit_safety_preamble = true;
    definition
}

#[test]
fn production_lifecycle_delivers_reports_and_refuses_invalid_context_before_admission() {
    // The production agent runner also uses a larger stack than libtest's
    // default. Keep its async lifecycle intact rather than mocking that path.
    std::thread::Builder::new()
        .name("coding-handoff-lifecycle".into())
        .stack_size(16 * 1024 * 1024)
        .spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("runtime")
                .block_on(check_production_lifecycle());
        })
        .expect("test thread")
        .join()
        .expect("lifecycle assertions");
}

async fn check_production_lifecycle() {
    let definition = definition();
    let workspace = tempfile::tempdir().expect("workspace");
    let model = Arc::new(RecordingModel::default());
    let mut invalid = SubagentRunOptions {
        task_id: Some("invalid-report".into()),
        context: Some(
            "[coding_handoff]\n{\"type\":\"coding_handoff\",\"version\":2,\"objective\":\"x\"}"
                .into(),
        ),
        checkpoint_dir: Some(workspace.path().join("checkpoints")),
        ..Default::default()
    };
    invalid.run_context.parent = Some(parent(model.clone(), workspace.path()));
    let result = run_subagent(&definition, "must not run", invalid).await;
    assert!(matches!(result, Err(SubagentRunError::InvalidContext(_))));
    assert!(model.requests.lock().is_empty());
    assert_eq!(std::fs::read_dir(workspace.path()).unwrap().count(), 0);

    let revision = "a".repeat(64);
    let packet = json!({
        "type": "coding_handoff", "version": 1, "objective": "Review Parser::parse",
        "source_refs": [{"path": "src/parser.rs", "revision": revision, "selected_symbols": ["Parser::parse"]}],
        "checks": [{"command": "touch should-not-exist", "cwd": workspace.path().display().to_string(), "status": "skipped"}],
    });
    for (task_id, context, expected) in [
        (
            "marked-report",
            format!("[coding_handoff]\n{packet}"),
            "[Coding handoff v1: untrusted report]",
        ),
        (
            "ordinary-context",
            "  ordinary context\nwith exact whitespace  ".into(),
            "  ordinary context\nwith exact whitespace  ",
        ),
    ] {
        let mut options = SubagentRunOptions {
            task_id: Some(task_id.into()),
            context: Some(context),
            checkpoint_dir: Some(workspace.path().join("checkpoints")),
            ..Default::default()
        };
        options.run_context.parent = Some(parent(model.clone(), workspace.path()));
        let outcome = run_subagent(&definition, "Report what remains unverified", options)
            .await
            .expect("run scripted child");
        assert_eq!(outcome.output, "Reviewed the supplied context");
        let requests = model.requests.lock();
        let user_text = requests
            .last()
            .expect("model call")
            .messages
            .iter()
            .filter(|message| matches!(message, Message::User(_)))
            .map(Message::text)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(user_text.contains(expected), "{user_text}");
        if task_id == "marked-report" {
            assert!(user_text.contains("Verification: unverified"));
            assert!(user_text.contains("Parser::parse"));
            assert!(user_text.contains(&revision));
            assert!(user_text.contains("touch should-not-exist"));
            assert!(user_text.contains("\"status\":\"skipped\""));
        }
    }
    assert!(!workspace.path().join("should-not-exist").exists());
    assert!(!workspace.path().join("src/parser.rs").exists());
}

#[tokio::test]
async fn public_spawn_and_delegate_tools_validate_before_model_or_workspace_work() {
    let malformed = "[coding_handoff]\nnot JSON";
    let tools: Vec<(Box<dyn Tool>, serde_json::Value)> = vec![
        (
            Box::new(SpawnSubagentTool::new()),
            json!({"agent_id": "critic", "prompt": "review", "context": malformed}),
        ),
        (
            Box::new(SpawnAsyncSubagentTool::new()),
            json!({"agent_id": "critic", "prompt": "review", "context": malformed, "fresh": true}),
        ),
        (
            Box::new(SpawnParallelAgentsTool::new()),
            json!({"tasks": [
                {"agent_id": "critic", "prompt": "review", "context": "ordinary"},
                {"agent_id": "critic", "prompt": "review", "context": malformed, "isolation": "worktree"}
            ]}),
        ),
        (
            Box::new(DelegateTool::new(
                HashMap::new(),
                Arc::new(SecurityPolicy::default()),
            )),
            json!({"agent": "not-configured", "prompt": "review", "context": malformed}),
        ),
    ];
    for (tool, arguments) in tools {
        let result = tool.execute(arguments).await.expect("tool result");
        assert!(result.is_error, "{} must reject", tool.name());
        assert!(
            result.output().contains("coding_handoff:"),
            "{}: {}",
            tool.name(),
            result.output()
        );
    }
}
