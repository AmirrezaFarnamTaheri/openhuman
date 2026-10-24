//! A loopback-only MCP server for registry lifecycle tests.
//!
//! Registry-installed stdio processes require a real native jail. Lifecycle
//! and routing tests use HTTP so they also run on CI hosts without that jail;
//! trusted-stdio and refusal tests cover process policy separately.

use serde_json::{json, Value};
use wiremock::{matchers::method, Mock, MockServer, Request, ResponseTemplate};

pub async fn echo_server() -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(|request: &Request| {
            let request: Value = serde_json::from_slice(&request.body).expect("MCP JSON request");
            let Some(id) = request.get("id") else {
                return ResponseTemplate::new(202);
            };
            let result = match request["method"].as_str().unwrap_or_default() {
                "initialize" => json!({
                    "protocolVersion": request["params"]["protocolVersion"],
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "openhuman-test-echo", "version": "1.0" }
                }),
                "ping" => json!({}),
                "tools/list" => json!({ "tools": [{
                    "name": "echo",
                    "description": "Return the message argument.",
                    "inputSchema": {
                        "type": "object",
                        "properties": { "message": { "type": "string" } },
                        "required": ["message"]
                    }
                }] }),
                "tools/call" if request["params"]["name"] == "echo" => json!({
                    "content": [{
                        "type": "text",
                        "text": request["params"]["arguments"]["message"].as_str().unwrap_or_default()
                    }],
                    "isError": false
                }),
                _ => {
                    return ResponseTemplate::new(200).set_body_json(json!({
                        "jsonrpc": "2.0", "id": id,
                        "error": { "code": -32601, "message": "Unknown MCP method or tool" }
                    }));
                }
            };
            ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0", "id": id, "result": result
            }))
        })
        .mount(&server)
        .await;
    // This stateless fixture answers requests directly; it has no server-push
    // event stream or session resources for the client to close.
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(405))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&server)
        .await;
    server
}
