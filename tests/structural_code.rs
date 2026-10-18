//! Structural tools through OpenHuman's real registry, filesystem policy and
//! coding pack. The finish policy is included from its owning module so this
//! integration target exercises the installed policy without copying it.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use openhuman_core::config::Config;
use openhuman_core::security::{AuditLogger, AutonomyLevel, SecurityPolicy};
use openhuman_core::tools::{ops, toolpacks};
use serde_json::{json, Value};
use tinyagents_harness::context::{RunConfig, RunContext};
use tinyagents_harness::ids::CallId;
use tinyagents_harness::tool::ToolExecutionContext;
use tinytools::{Tool, ToolCallOptions, ToolResult, WorkspaceDescriptor};

#[path = "structural_code/catalog_tests.rs"]
mod catalog_tests;
#[path = "structural_code/filesystem_tests.rs"]
mod filesystem_tests;
#[path = "../crates/openhuman-core/src/agent/tinyagents/verify_before_finish.rs"]
mod finish_policy;
#[path = "structural_code/harness_tests.rs"]
mod harness_tests;

const SOURCE: &str = "fn value() -> i32 { 1 }\n";
const CHUNK_ID: &str = "rust.fallible-function.v1";
const READ_TOOLS: [&str; 4] = ["ast_outline", "ast_symbol", "chunk_search", "chunk_inspect"];
const WRITE_TOOLS: [&str; 2] = ["ast_patch", "chunk_apply"];

#[derive(Clone, Copy, Debug)]
enum Route {
    Direct,
    Packed,
}

impl Route {
    fn call(self, name: &str, args: Value) -> (String, Value) {
        match self {
            Self::Direct => (name.to_owned(), args),
            Self::Packed => (
                "use_skill".to_owned(),
                json!({"skill": "coding", "tool": name, "args": args}),
            ),
        }
    }
}

struct Fixture {
    _root: tempfile::TempDir,
    home: PathBuf,
    project: PathBuf,
    security: Arc<SecurityPolicy>,
    tools: Arc<Vec<Box<dyn Tool>>>,
}

impl Fixture {
    fn new(autonomy: AutonomyLevel) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("host");
        let project = root.path().join("project");
        let action = home.join("action");
        std::fs::create_dir_all(&action).unwrap();
        std::fs::create_dir(&project).unwrap();
        std::fs::write(project.join("source.rs"), SOURCE).unwrap();
        let config = Arc::new(Config {
            workspace_dir: home.clone(),
            action_dir: action.clone(),
            config_path: home.join("config.toml"),
            ..Config::default()
        });
        let security = Arc::new(SecurityPolicy {
            enabled: true,
            autonomy,
            workspace_dir: home.clone(),
            action_dir: action.clone(),
            workspace_only: true,
            ..SecurityPolicy::default()
        });
        let tools = Arc::new(ops::all_tools(
            config.clone(),
            &security,
            AuditLogger::disabled(),
            &config.browser,
            &config.http_request,
            &action,
            &HashMap::new(),
            &config,
        ));
        toolpacks::bind_pack_registry(&tools);
        Self {
            _root: root,
            home,
            project,
            security,
            tools,
        }
    }

    fn tool(&self, name: &str) -> &dyn Tool {
        self.tools
            .iter()
            .find(|tool| tool.name() == name)
            .unwrap()
            .as_ref()
    }

    fn run_context_at(root: &Path) -> RunContext<()> {
        RunContext::new(RunConfig::new("structural-code"), ()).with_workspace(
            WorkspaceDescriptor::new(root.to_path_buf()).with_policy_id("structural-code-test"),
        )
    }

    fn run_context(&self) -> RunContext<()> {
        Self::run_context_at(&self.project)
    }

    async fn call(&self, route: Route, name: &str, args: Value) -> ToolResult {
        let context =
            ToolExecutionContext::from_run_context(&self.run_context(), CallId::new("call"));
        let (outer, args) = route.call(name, args);
        self.tool(&outer)
            .execute_with_context(args, ToolCallOptions::default(), Some(&context))
            .await
            .unwrap()
    }

    async fn json(&self, route: Route, name: &str, args: Value) -> Value {
        let result = self.call(route, name, args).await;
        assert!(!result.is_error, "{route:?} {name}: {}", result.output());
        serde_json::from_str(&result.output()).unwrap()
    }

    async fn mutation_args(&self, name: &str, preview: bool) -> Value {
        match name {
            "ast_patch" => {
                let outline = self
                    .json(Route::Direct, "ast_outline", json!({"path": "source.rs"}))
                    .await;
                json!({
                    "path": "source.rs", "expected_revision": outline["revision"],
                    "selector": "value", "operation": "replace_body", "code": "2", "dry_run": preview
                })
            }
            "chunk_apply" => {
                let inspected = self
                    .json(Route::Direct, "chunk_inspect", json!({"id": CHUNK_ID}))
                    .await;
                json!({
                    "id": CHUNK_ID, "path": "new.rs", "create_new": true,
                    "chunk_revision": inspected["chunk_revision"],
                    "bindings": {"parameters": {"name": "created"}}, "dry_run": preview
                })
            }
            _ => panic!("not a structural mutator: {name}"),
        }
    }

    fn assert_original_files(&self) {
        assert_eq!(
            std::fs::read_to_string(self.project.join("source.rs")).unwrap(),
            SOURCE
        );
        assert!(!self.project.join("new.rs").exists());
        assert_eq!(std::fs::read_dir(&self.project).unwrap().count(), 1);
    }
}
