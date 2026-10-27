//! Scoped async preparation for typed sub-agent prompt rendering.

use super::{AgentDefinition, PromptContext, SubagentRunError};
use crate::agent::subagent_host::tool_prep::load_prompt_source;
use crate::agent_registry::agents::orchestrator as orchestrator_prompt;
use crate::agent_registry::host_definition::AgentDefinitionExt;
use crate::core::runtime::DispatchCtx;

/// One spawn's config snapshot, loaded once by
/// [`super::run_subagent_direct_with_native_context`].
///
/// The config loader is not cached. Reusing this snapshot keeps prompt, model
/// and context decisions on the same profile if the file changes mid-spawn.
/// Preserve load failures as strings so callers can report the cause while
/// retaining their existing fallback behavior.
pub(super) type LoadedConfig = Result<std::sync::Arc<crate::config::Config>, String>;

/// Load and render a typed sub-agent prompt using this spawn's config snapshot.
pub(super) async fn load_typed_prompt_source(
    definition: &AgentDefinition,
    _config: &LoadedConfig,
    mut prompt_context: PromptContext<'_>,
    dispatch: &DispatchCtx,
) -> Result<String, SubagentRunError> {
    let mcp_available = definition.prompt_needs_connected_mcp()
        && orchestrator_prompt::mcp_tools_reachable(
            prompt_context.visible_tool_names,
            prompt_context.tools.iter().map(|tool| tool.name.as_ref()),
        );
    if mcp_available {
        prompt_context.connected_mcp_servers_md =
            orchestrator_prompt::render_connected_mcp_servers_async(dispatch).await;
    }

    #[cfg(feature = "flows")]
    let flow_archetype_text = crate::agent::prompts::flow_archetype::load_for_definition(
        _config.as_ref().ok().map(|config| config.as_ref()),
        definition,
    )
    .await?;
    #[cfg(not(feature = "flows"))]
    let flow_archetype_text: Option<String> = None;

    prompt_context.flow_archetype_text = flow_archetype_text.as_deref();
    load_prompt_source(definition, &prompt_context)
}
