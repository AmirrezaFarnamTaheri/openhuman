//! OpenHuman spawn-hierarchy admission policy.

use crate::agent_registry::host_definition::AgentDefinition;
use crate::agent::subagent_host::SubagentRunError;
use tinyagents_definition::{validate_tier_transition, AgentTier};

/// Runtime spawn-hierarchy gate decision for one delegation hop.
///
/// `parent_def` is the resolved parent agent definition (looked up from the
/// global registry by its definition id) or `None` when the parent can't be
/// resolved — e.g. a dynamically-named agent (model-council juror) or a custom
/// agent absent from the registry, or any context where the registry isn't
/// initialised. A `None` parent yields `Ok(())`: we skip rather than mask, the
/// same defensive posture the loader takes for unknown child ids.
///
/// A **worker** parent is also exempted. A worker's `subagents` list holds no
/// agent id (the loader rejects one), so any spawn it reaches at runtime is
/// one the host dispatched for it. Re-denying it here would turn valid custom
/// worker agents that use `{ skills = "*" }` into runtime failures. The
/// worker-leaf authoring rule stays enforced statically at boot, and the
/// per-parent allowlist gate blocks any other worker spawn.
///
/// For chat / reasoning parents the hop is checked against
/// [`validate_tier_transition`] (the single source of truth shared with the
/// boot loader walk); a forbidden hop is logged and becomes a
/// [`SubagentRunError::TierViolation`]. Logging lives here (rather than at the
/// call site) so the deny path is exercised by this fn's unit tests.
pub(in super::super) fn tier_gate_decision(
    parent_def: Option<&AgentDefinition>,
    child: &AgentDefinition,
    parent_agent_id: &str,
    task_id: &str,
) -> Result<(), SubagentRunError> {
    let Some(parent_def) = parent_def else {
        return Ok(());
    };
    if parent_def.agent_tier == AgentTier::Worker {
        return Ok(());
    }
    if let Err(reason) = validate_tier_transition(parent_def.agent_tier, child.agent_tier) {
        tracing::warn!(
            parent_agent = %parent_agent_id,
            parent_tier = %parent_def.agent_tier,
            child_agent = %child.id,
            child_tier = %child.agent_tier,
            task_id = %task_id,
            "[subagent_host] blocked tier-violating delegation: {reason}"
        );
        return Err(SubagentRunError::TierViolation {
            parent_tier: parent_def.agent_tier,
            child_tier: child.agent_tier,
            reason,
        });
    }
    Ok(())
}
