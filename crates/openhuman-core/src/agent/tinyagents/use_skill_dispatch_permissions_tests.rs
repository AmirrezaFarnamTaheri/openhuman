use super::*;

use serde_json::json;
use tinyagents_harness::tool::packs::UseSkillTool;
use tinytools::PermissionLevel;

struct MixedCodeTool;

#[async_trait]
impl Tool for MixedCodeTool {
    fn name(&self) -> &str {
        "ast_patch"
    }

    fn description(&self) -> &str {
        "A preview-or-write target used to exercise the host dispatch boundary."
    }

    fn parameters_schema(&self) -> Value {
        json!({"type": "object"})
    }

    fn permission_level(&self) -> PermissionLevel {
        PermissionLevel::ReadOnly
    }

    fn permission_level_with_args(&self, args: &Value) -> PermissionLevel {
        if args.get("dry_run") == Some(&Value::Bool(true)) {
            PermissionLevel::ReadOnly
        } else {
            PermissionLevel::Write
        }
    }

    fn external_effect_with_args(&self, args: &Value) -> bool {
        args.get("publish") == Some(&Value::Bool(true))
    }

    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        Ok(ToolResult::success(args.to_string()))
    }
}

fn dispatch_with_target(include_target: bool) -> (UseSkillDispatch, PackRegistryHandle) {
    let handle = PackRegistryHandle::default();
    let mut tools: Vec<Box<dyn Tool>> = vec![Box::new(UseSkillTool::new(handle.clone(), CATALOG))];
    if include_target {
        tools.push(Box::new(MixedCodeTool));
    }
    let tools = Arc::new(tools);
    handle.bind(Arc::downgrade(&tools));
    let adapter = super::super::tool_registration::shared_tool_adapter(vec![tools], "use_skill")
        .expect("registered dispatcher");
    (
        UseSkillDispatch::new(Arc::new(adapter), handle.clone()),
        handle,
    )
}

#[test]
fn missing_named_host_calls_keep_the_permission_ceiling_after_binding() {
    let (dispatch, _) = dispatch_with_target(true);
    assert_eq!(
        dispatch.tool().permission_level(),
        PermissionLevel::ReadOnly
    );
    for args in [
        json!({"skill":"coding", "tool":"missing"}),
        json!({"skill":"media", "tool":"ast_patch"}),
        json!({"skill":"missing", "tool":"ast_patch"}),
    ] {
        let bound = dispatch
            .bind_call(&args)
            .unwrap()
            .expect("named call binds");
        assert_eq!(bound.tool().permission_level(), PermissionLevel::Dangerous);
        assert_eq!(
            bound
                .tool()
                .permission_level_with_args(&json!({"skill":"coding"})),
            PermissionLevel::Dangerous,
            "a later disclosure-shaped wrapper argument cannot lower the frozen answer"
        );
        assert!(bound.admission_target().is_none());
    }
    let disclosure = json!({"skill":"coding"});
    assert!(dispatch.bind_call(&disclosure).unwrap().is_none());
    assert_eq!(
        dispatch.tool().permission_level_with_args(&disclosure),
        PermissionLevel::ReadOnly
    );
}

#[tokio::test]
async fn a_missing_host_target_cannot_appear_after_authorization() {
    let (dispatch, handle) = dispatch_with_target(false);
    let args = json!({"skill":"coding", "tool":"ast_patch", "args":{"dry_run":true}});
    let bound = dispatch.bind_call(&args).unwrap().unwrap();
    let replacement: Arc<Vec<Box<dyn Tool>>> = Arc::new(vec![Box::new(MixedCodeTool)]);
    handle.bind(Arc::downgrade(&replacement));
    assert_eq!(
        bound.tool().permission_level_with_args(&args),
        PermissionLevel::Dangerous
    );
    let result = bound.tool().execute(args).await.unwrap();
    assert!(
        result.is_error,
        "the frozen missing-target answer must not execute the replacement"
    );
}

#[tokio::test]
async fn the_host_keeps_selected_target_permissions_effects_and_arguments_pinned() {
    let (dispatch, _) = dispatch_with_target(true);
    for preview in [true, false] {
        let inner = json!({"dry_run":preview, "publish":!preview});
        let args = json!({"skill":"coding", "tool":"ast_patch", "args":inner});
        let bound = dispatch.bind_call(&args).unwrap().unwrap();
        let rewritten = json!({"skill":"coding", "tool":"ast_patch", "args":{"dry_run":!preview, "publish":preview}});
        let expected = if preview {
            PermissionLevel::ReadOnly
        } else {
            PermissionLevel::Write
        };
        assert_eq!(
            bound.tool().permission_level_with_args(&rewritten),
            expected
        );
        assert_eq!(bound.tool().external_effect_with_args(&rewritten), !preview);
        let (target, observed) = bound
            .admission_target()
            .expect("actual target is available");
        assert_eq!(target.name(), "ast_patch");
        assert_eq!(observed, &inner);
        assert_eq!(target.permission_level_with_args(observed), expected);
        let result = bound.tool().execute(rewritten).await.unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(&result.output()).unwrap(),
            inner
        );
    }
}
