use super::*;

use tinyagents_definition::{SandboxMode, ToolScope};
use tinytools::{PermissionLevel, ToolExposure};

#[test]
fn the_full_registry_and_coding_catalog_resolve_all_six_tools_once() {
    let fixture = Fixture::new(AutonomyLevel::Full);
    for name in READ_TOOLS.into_iter().chain(WRITE_TOOLS) {
        assert_eq!(
            fixture
                .tools
                .iter()
                .filter(|tool| tool.name() == name)
                .count(),
            1,
            "{name}"
        );
        let tool = fixture.tool(name);
        assert_eq!(tool.exposure(), ToolExposure::Deferred);
        assert_eq!(tool.permission_level(), PermissionLevel::ReadOnly);
        assert_eq!(toolpacks::pack_for_tool(name).unwrap().id, "coding");
        assert_eq!(tool.parameters_schema()["type"], "object");
    }
    for name in WRITE_TOOLS {
        assert_eq!(
            fixture
                .tool(name)
                .permission_level_with_args(&json!({"dry_run": true})),
            PermissionLevel::ReadOnly
        );
        for args in [
            json!({}),
            json!({"dry_run": false}),
            json!({"dry_run": "true"}),
        ] {
            assert_eq!(
                fixture.tool(name).permission_level_with_args(&args),
                PermissionLevel::Write
            );
        }
    }
    let lightweight: HashSet<_> = ops::default_tools(fixture.security.clone())
        .iter()
        .map(|tool| tool.name().to_owned())
        .collect();
    assert_eq!(
        lightweight,
        ["shell", "file_read", "file_write"]
            .map(str::to_owned)
            .into()
    );
}

#[test]
fn planner_and_critic_keep_only_the_four_structural_readers_visible() {
    let definitions = openhuman_core::agent::registry::agents::load_builtins().unwrap();
    for id in ["planner", "critic"] {
        let definition = definitions
            .iter()
            .find(|definition| definition.id == id)
            .unwrap();
        assert_eq!(definition.sandbox_mode, SandboxMode::ReadOnly);
        let ToolScope::Named(names) = &definition.tools else {
            panic!("{id} must have an explicit tool scope");
        };
        let mut visible: HashSet<String> = names.iter().cloned().collect();
        toolpacks::strip_packed_from_visible(&mut visible, id);
        for name in READ_TOOLS {
            assert!(visible.contains(name), "{id} is missing {name}");
        }
        for name in WRITE_TOOLS {
            assert!(!visible.contains(name), "{id} must not receive {name}");
        }
    }
    let mut orchestrator: HashSet<String> = READ_TOOLS
        .into_iter()
        .chain(WRITE_TOOLS)
        .map(str::to_owned)
        .collect();
    toolpacks::strip_packed_from_visible(&mut orchestrator, "orchestrator");
    assert_eq!(orchestrator, ["use_skill".to_owned()].into());
}

#[tokio::test]
async fn pack_disclosure_and_readers_expose_real_bounded_results() {
    let fixture = Fixture::new(AutonomyLevel::ReadOnly);
    let disclosed = fixture
        .tool("use_skill")
        .execute(json!({"skill": "coding"}))
        .await
        .unwrap();
    assert!(!disclosed.is_error, "{}", disclosed.output());
    for name in READ_TOOLS.into_iter().chain(WRITE_TOOLS) {
        assert!(
            disclosed.output().contains(name),
            "coding disclosure is missing {name}"
        );
    }
    let outline = fixture
        .json(
            Route::Packed,
            "ast_outline",
            json!({"path": "source.rs", "level": 0}),
        )
        .await;
    assert!(outline.get("skeleton").is_none());
    let symbol = fixture
        .json(
            Route::Packed,
            "ast_symbol",
            json!({"path": "source.rs", "selector": "value"}),
        )
        .await;
    assert_eq!(symbol["revision"], outline["revision"]);
    assert_eq!(symbol["symbol"]["name"], "value");
    assert!(symbol["source"]["text"]
        .as_str()
        .unwrap()
        .contains("fn value"));
    let skeleton = fixture
        .json(
            Route::Packed,
            "ast_outline",
            json!({"path": "source.rs", "level": 1}),
        )
        .await;
    assert!(skeleton["skeleton"]["text"]
        .as_str()
        .unwrap()
        .contains("value"));
    let search = fixture.json(Route::Packed, "chunk_search", json!({})).await;
    assert_eq!(search["chunks"]["items"].as_array().unwrap().len(), 5);
    let inspected = fixture
        .json(Route::Packed, "chunk_inspect", json!({"id": CHUNK_ID}))
        .await;
    assert_eq!(inspected["id"], CHUNK_ID);
    assert_eq!(inspected["chunk_revision"].as_str().unwrap().len(), 64);
    fixture.assert_original_files();
}
