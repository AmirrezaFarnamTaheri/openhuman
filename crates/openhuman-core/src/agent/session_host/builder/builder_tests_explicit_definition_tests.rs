//! `from_config_with_definition` and `resolved_definition`: an explicit
//! definition is authoritative for the session it builds, and the in-turn
//! readers (`sandbox_mode`, `subagents`) consult it before the process
//! registry.

use super::*;
use crate::agent::session_host::types::OpenHumanSessionHost;
use crate::agent_registry::host_definition::{AgentDefinitionExt, AgentDefinitionRegistry};
use tinyagents_definition::{PromptSource, SandboxMode, ToolScope};

fn explicit_definition(id: &str) -> crate::agent_registry::host_definition::AgentDefinition {
    let mut def = AgentDefinitionRegistry::builtins_only()
        .get("orchestrator")
        .cloned()
        .expect("built-in orchestrator");
    def.id = id.to_string();
    def.display_name = Some("Explicit".to_string());
    def.system_prompt = PromptSource::Inline("You are explicit.".to_string());
    def.tools = ToolScope::Wildcard;
    // The built-in orchestrator runs `Sandboxed`; `ReadOnly` is observably
    // different at every reader that consults the definition.
    def.sandbox_mode = SandboxMode::ReadOnly;
    def
}

#[tokio::test]
async fn from_config_with_definition_stamps_the_definition_on_the_session() {
    let tmp = tempfile::TempDir::new().unwrap();
    let config = test_config(&tmp);
    let def = explicit_definition("embedded-alpha");

    let agent = OpenHumanSessionHost::from_config_with_definition(
        &crate::core::runtime::DispatchCtx::ambient(),
        &config,
        &def,
    )
    .expect("build from an explicit definition");

    assert_eq!(agent.agent_definition_id, "embedded-alpha");
    let resolved = agent
        .resolved_definition()
        .expect("the explicit definition is stamped");
    assert_eq!(resolved.id, "embedded-alpha");
    assert_eq!(resolved.sandbox_mode, SandboxMode::ReadOnly);
    assert!(
        matches!(resolved.system_prompt, PromptSource::Inline(ref p) if p == "You are explicit.")
    );
}

#[tokio::test]
async fn resolved_definition_prefers_the_sessions_own_over_a_same_id_registry_entry() {
    // A library host may reuse an id the process registry also knows. The
    // session's own definition must win at the security-relevant reads
    // (`sandbox_mode`), never the registry's copy.
    let _ = AgentDefinitionRegistry::init_global_builtins();
    let tmp = tempfile::TempDir::new().unwrap();
    let config = test_config(&tmp);
    let def = explicit_definition("orchestrator");
    assert_ne!(
        AgentDefinitionRegistry::global()
            .and_then(|r| r.get("orchestrator"))
            .map(|d| d.sandbox_mode),
        Some(SandboxMode::ReadOnly),
        "the registry's orchestrator is not read-only, so a leak is observable"
    );

    let agent = OpenHumanSessionHost::from_config_with_definition(
        &crate::core::runtime::DispatchCtx::ambient(),
        &config,
        &def,
    )
    .expect("build from an explicit definition");

    assert_eq!(
        agent.resolved_definition().map(|d| d.sandbox_mode),
        Some(SandboxMode::ReadOnly)
    );
}

#[tokio::test]
async fn registry_built_sessions_resolve_the_registry_definition() {
    // Byte-identical behaviour for every existing caller: a session built by
    // id carries the registry's definition and reads it back.
    let _ = AgentDefinitionRegistry::init_global_builtins();
    let tmp = tempfile::TempDir::new().unwrap();
    let config = test_config(&tmp);

    let agent =
        OpenHumanSessionHost::from_config(&crate::core::runtime::DispatchCtx::ambient(), &config)
            .expect("orchestrator session");
    let resolved = agent.resolved_definition().expect("registry definition");
    assert_eq!(resolved.id, "orchestrator");
    let registry_mode = AgentDefinitionRegistry::global()
        .and_then(|r| r.get("orchestrator"))
        .map(|d| d.sandbox_mode)
        .expect("registry orchestrator");
    assert_eq!(resolved.sandbox_mode, registry_mode);
}

#[tokio::test]
async fn mcp_snapshot_follows_the_selected_prompt_for_a_renamed_builtin() {
    let tmp = tempfile::TempDir::new().unwrap();
    let config = test_config(&tmp);
    let mut definition = AgentDefinitionRegistry::builtins_only()
        .get("orchestrator")
        .cloned()
        .expect("built-in orchestrator");
    definition.id = "renamed-mcp-prompt".to_string();
    // Worker conversion changes tool policy while retaining the prompt source.
    definition.extensions.searches_connected_mcp = false;
    let dynamic = definition.system_prompt.clone();
    for (source, expected) in [
        (dynamic, true),
        (PromptSource::Inline("Custom prompt.".to_string()), false),
        (PromptSource::Verbatim("Exact prompt.".to_string()), false),
    ] {
        definition.system_prompt = source;
        assert_eq!(definition.prompt_needs_connected_mcp(), expected);
        let agent = OpenHumanSessionHost::from_config_with_definition(
            &crate::core::runtime::DispatchCtx::ambient(),
            &config,
            &definition,
        )
        .expect("renamed builtin builds with its selected prompt");
        assert_eq!(
            agent.prompt_builder.needs_connected_mcp_snapshot(),
            expected,
            "only the selected MCP-aware prompt may request a snapshot"
        );
    }
}
