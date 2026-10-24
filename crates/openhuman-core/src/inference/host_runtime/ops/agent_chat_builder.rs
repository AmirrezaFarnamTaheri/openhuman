//! Keep synchronous session construction off the executor that drives its callbacks.

use super::{definition_for_turn, AgentChatTarget, Config, OpenHumanSessionHost};
use crate::agent::HostTools;
use crate::agent_registry::host_definition::AgentDefinition;
use crate::core::runtime::DispatchCtx;

enum BuildTarget {
    Orchestrator,
    AgentId(String),
    Definition {
        definition: Box<AgentDefinition>,
        host: Option<HostTools>,
        host_only: bool,
    },
}

impl BuildTarget {
    fn build(
        self,
        ctx: &DispatchCtx,
        config: &Config,
        session_id: Option<&str>,
    ) -> Result<OpenHumanSessionHost, String> {
        match self {
            Self::Orchestrator => OpenHumanSessionHost::from_config(ctx, config),
            Self::AgentId(id) => {
                log::debug!("[inference] agent_chat building agent_id={id}");
                OpenHumanSessionHost::from_config_for_agent(ctx, config, &id)
            }
            Self::Definition {
                definition,
                host,
                host_only: true,
            } => OpenHumanSessionHost::from_config_host_only(
                ctx,
                config,
                &definition,
                host.as_ref(),
                session_id,
            ),
            Self::Definition {
                definition, host, ..
            } => match host {
                Some(host) => OpenHumanSessionHost::from_config_with_host_tools(
                    ctx,
                    config,
                    &definition,
                    &host,
                    session_id,
                ),
                None => OpenHumanSessionHost::from_config_with_definition(ctx, config, &definition),
            },
        }
        .map_err(|error| error.to_string())
    }
}

pub(super) async fn build_turn_agent(
    ctx: &DispatchCtx,
    config: &Config,
    target: &AgentChatTarget<'_>,
    session_id: Option<&str>,
    temperature: Option<f64>,
    construction_keepalive: Option<std::sync::Arc<dyn Send + Sync>>,
) -> Result<OpenHumanSessionHost, String> {
    let target = match target {
        AgentChatTarget::Orchestrator => BuildTarget::Orchestrator,
        AgentChatTarget::AgentId(id) => BuildTarget::AgentId((*id).to_owned()),
        AgentChatTarget::Definition {
            definition,
            host,
            host_only,
            ..
        } => BuildTarget::Definition {
            definition: Box::new(definition_for_turn(definition, temperature).into_owned()),
            host: host.cloned(),
            host_only: *host_only,
        },
    };
    let config = config.clone();
    let session_id = session_id.map(str::to_owned);
    let build_context = ctx
        .turn_origin_or_scoped()
        .map_or_else(|| ctx.clone(), |origin| ctx.with_turn_origin(origin));
    // Provider sources retain a weak binding to the caller's long-lived
    // context; the derived origin scope is only for this blocking task.
    let task_context = ctx.clone();
    // Tool and workflow catalogues use synchronous storage/module bridges.
    // Blocking an executor worker here can starve a concurrent initializer
    // those bridges await, even after block_in_place hands off the worker.
    // Carry the same context into the blocking pool and let the caller yield.
    let (agent, _construction_keepalive) = build_context
        .spawn_blocking(move || {
            let agent = target.build(&task_context, &config, session_id.as_deref());
            // A detached task drops the built session before its resource
            // lease. A cancelled caller must not purge a live constructor.
            (agent, construction_keepalive)
        })
        .await
        .map_err(|error| format!("agent session construction task failed: {error}"))?;
    agent
}
