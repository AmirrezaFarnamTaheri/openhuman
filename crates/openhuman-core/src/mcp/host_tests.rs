//! Tests for OpenHuman's product-to-module MCP configuration mapping.

use super::*;
use crate::config::{HttpHeader, McpAuthConfig, McpServerConfig as HostServer};

fn populated_server() -> HostServer {
    HostServer {
        server: tinymcp_bus::McpServerConfig {
            name: "weather".into(),
            endpoint: "https://example.test/mcp".into(),
            command: "npx".into(),
            args: vec!["-y".into(), "weather-mcp".into()],
            env: [("API_KEY".to_string(), "secret".to_string())].into(),
            cwd: Some("/workspace".into()),
            description: Some("Weather lookups".into()),
            enabled: false,
            allowed_tools: vec!["forecast".into()],
            disallowed_tools: vec!["debug".into()],
            operator_trusted: false,
            timeout_secs: 9,
            auth: McpAuthConfig::BearerToken { token: "t".into() },
        },
        expose: Default::default(),
        direct_tools: Vec::new(),
    }
}

#[test]
fn declared_server_fields_and_identity_survive_conversion() {
    let mut config = Config::default();
    config.mcp_client.servers.push(populated_server());
    config.mcp_client.client_identity.name = "openhuman-core".into();
    config.mcp_client.client_identity.title = "OpenHuman Core MCP Client".into();
    config.mcp_client.client_identity.version = "9.9.9".into();
    config.mcp_client.registry_auth.smithery_api_key = Some("smithery".into());

    let converted = client_config(&config);
    let server = converted.servers.first().expect("the server");
    assert_eq!(server.name, "weather");
    assert_eq!(server.endpoint, "https://example.test/mcp");
    assert_eq!(server.command, "npx");
    assert_eq!(server.args, ["-y", "weather-mcp"]);
    assert_eq!(
        server.env.get("API_KEY").map(String::as_str),
        Some("secret")
    );
    assert_eq!(server.cwd.as_deref(), Some("/workspace"));
    assert_eq!(server.description.as_deref(), Some("Weather lookups"));
    assert!(!server.enabled);
    assert_eq!(server.allowed_tools, ["forecast"]);
    assert_eq!(server.disallowed_tools, ["debug"]);
    assert_eq!(server.timeout_secs, 9);
    assert_eq!(
        server.auth,
        McpAuthConfig::BearerToken { token: "t".into() }
    );
    assert_eq!(converted.client_identity.name, "openhuman-core");
    assert_eq!(converted.client_identity.title, "OpenHuman Core MCP Client");
    assert_eq!(converted.client_identity.version, "9.9.9");
    assert_eq!(
        converted.registry_auth.smithery_api_key.as_deref(),
        Some("smithery")
    );
}

#[test]
fn every_credential_kind_converts_without_shape_changes() {
    let cases = [
        McpAuthConfig::None,
        McpAuthConfig::BearerToken { token: "t".into() },
        McpAuthConfig::Basic {
            username: "u".into(),
            password: "p".into(),
        },
        McpAuthConfig::Header {
            name: "X-Key".into(),
            value: "v".into(),
        },
        McpAuthConfig::QueryParam {
            name: "key".into(),
            value: "v".into(),
        },
        McpAuthConfig::Headers {
            headers: vec![
                HttpHeader {
                    name: "X-Key".into(),
                    value: "k".into(),
                },
                HttpHeader {
                    name: "Authorization".into(),
                    value: "Bearer s".into(),
                },
            ],
        },
    ];

    for auth in cases {
        let mut config = Config::default();
        let mut server = populated_server();
        server.auth = auth.clone();
        config.mcp_client.servers.push(server);
        assert_eq!(client_config(&config).servers[0].auth, auth);
    }
}

#[test]
fn credentialed_plaintext_remote_servers_are_not_passed_to_the_module() {
    let mut config = Config::default();
    let mut server = populated_server();
    server.endpoint = "http://remote.example.test/mcp".into();
    config.mcp_client.servers.push(server);
    assert!(client_config(&config).servers.is_empty());

    let mut server = populated_server();
    server.endpoint = "http://127.0.0.1:8080/mcp".into();
    config.mcp_client.servers.push(server);
    assert_eq!(client_config(&config).servers.len(), 1);
}

#[test]
fn oauth_redirect_port_parser_accepts_only_explicit_url_ports() {
    assert_eq!(explicit_port("http://127.0.0.1:7789/rpc"), Some(7789));
    assert_eq!(explicit_port("http://127.0.0.1/rpc"), None);
    assert_eq!(explicit_port("not a URL"), None);
}

/// The config comes from the dispatch context passed in, not from the calling
/// task's scope: two agents resolved concurrently, across awaits and on spawned
/// tasks, each get their own, and an unbound context gets neither.
#[tokio::test]
async fn ambient_config_follows_the_explicit_context() {
    use crate::core::runtime::{ContextOverlay, CoreContext, DispatchCtx, DomainSet};
    let agent = |name: &str| {
        let config = Config {
            workspace_dir: std::path::PathBuf::from(format!("/tmp/mcp-host-{name}")),
            ..Config::default()
        };
        DispatchCtx::new(CoreContext::for_test(DomainSet::full(), None).derive_with(
            ContextOverlay::new(config, DomainSet::full(), Default::default()).session_agent(name),
        ))
    };
    let (alice, bob) = (agent("alice"), agent("bob"));
    let resolve = |ctx: DispatchCtx| {
        tokio::spawn(async move {
            tokio::task::yield_now().await;
            ambient_config(&ctx).map(|config| config.workspace_dir)
        })
    };
    let (a, b) = tokio::join!(resolve(alice), resolve(bob));
    assert_eq!(
        a.unwrap().as_deref(),
        Some(std::path::Path::new("/tmp/mcp-host-alice"))
    );
    assert_eq!(
        b.unwrap().as_deref(),
        Some(std::path::Path::new("/tmp/mcp-host-bob"))
    );
    assert_eq!(
        ambient_config(&DispatchCtx::unbound()).map(|config| config.workspace_dir),
        DEFAULT_CONFIG
            .get()
            .map(|config| config.workspace_dir.clone()),
        "an unbound context falls back to the process default, never a scope"
    );
}

#[cfg(feature = "modules")]
#[test]
fn installed_stdio_servers_preserve_the_host_workspace_jail_and_grants() {
    let workspace_a = tempfile::tempdir().expect("workspace A");
    let workspace_b = tempfile::tempdir().expect("workspace B");
    let mut config = Config::default();
    config.workspace_dir = workspace_a.path().to_path_buf();

    let jail_a = client_config(&config)
        .stdio_sandbox
        .expect("untrusted stdio needs a jail");
    assert_eq!(
        jail_a.workspace,
        workspace_a
            .path()
            .join("mcp_stdio")
            .canonicalize()
            .unwrap()
            .to_string_lossy(),
    );
    assert_eq!(jail_a.require_process_isolation, !cfg!(target_os = "macos"));
    assert!(
        jail_a.allow_subprocess,
        "stdio wrappers may need a child process"
    );
    assert!(
        jail_a.allow_net,
        "remote MCP backends may need network access"
    );
    assert!(jail_a.read_only.is_empty());
    assert!(jail_a.read_write.is_empty());
    if jail_a.require_process_isolation {
        assert!(jail_a.deny_read.is_empty());
    }

    config.workspace_dir = workspace_b.path().to_path_buf();
    let jail_b = client_config(&config)
        .stdio_sandbox
        .expect("second workspace also needs a jail");
    assert_eq!(
        jail_b.workspace,
        workspace_b
            .path()
            .join("mcp_stdio")
            .canonicalize()
            .unwrap()
            .to_string_lossy(),
    );
    assert_ne!(
        jail_a.workspace, jail_b.workspace,
        "do not reuse another workspace's jail"
    );
    assert_eq!(jail_b.require_process_isolation, !cfg!(target_os = "macos"));
}

#[cfg(feature = "modules")]
#[test]
fn configured_stdio_grants_survive_client_conversion() {
    let workspace = tempfile::tempdir().unwrap();
    let tools = workspace.path().join("tools");
    std::fs::create_dir(&tools).unwrap();
    let tools = tools.canonicalize().unwrap().to_string_lossy().into_owned();
    let mut config = Config::default();
    config.workspace_dir = workspace.path().to_path_buf();
    config.runtime.local_jail.extra_read_only = vec![tools.clone()];
    let sandbox = client_config(&config).stdio_sandbox.expect("host jail");
    assert_eq!(sandbox.read_only, vec![tools]);
    assert!(sandbox.read_write.is_empty());
}
