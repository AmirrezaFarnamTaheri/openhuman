// --- host-registered agents (agent::host_agents) ---

struct FlowsHost;

impl crate::agent::host_agents::HostAgentResolver for FlowsHost {
    fn resolve(&self, agent_id: &str) -> Option<crate::agent::host_agents::HostAgent> {
        if agent_id != "flows-host-agent" {
            return None;
        }
        let mut definition =
            crate::agent_registry::host_definition::AgentDefinitionRegistry::builtins_only()
                .get("orchestrator")
                .cloned()
                .unwrap();
        definition.id = agent_id.to_string();
        Some(crate::agent::host_agents::HostAgent {
            definition,
            config: crate::config::Config::default(),
            host_tools: None,
            hooks: Default::default(),
            context: crate::core::runtime::CoreContext::for_test(
                crate::core::runtime::DomainSet::full(),
                None,
            ),
        })
    }
}

#[test]
fn a_host_registered_agent_routes_ahead_of_the_registries() {
    let _slot = crate::agent::host_agents::tests::lock();
    let installed: std::sync::Arc<dyn crate::agent::host_agents::HostAgentResolver> =
        std::sync::Arc::new(FlowsHost);
    crate::agent::host_agents::install(std::sync::Arc::clone(&installed));
    let host = super::route_for_agent_ref("flows-host-agent");
    let other = super::route_for_agent_ref("flows-not-a-host-agent");
    crate::agent::host_agents::clear_if(&installed);
    assert_eq!(host, super::AgentRoute::HostAgent);
    assert_eq!(other, super::AgentRoute::RegistryFallback);
}
