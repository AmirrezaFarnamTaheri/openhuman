use super::*;
use crate::config::schema::cloud_providers::ToolCallCodec;

#[test]
fn config_domain_provider_update_accepts_all_supported_tool_call_codecs() {
    for (wire, mode) in [
        ("native", ToolCallCodec::Native),
        ("qnml", ToolCallCodec::Qnml),
        ("tagged_json", ToolCallCodec::TaggedJson),
    ] {
        let provider: CloudProviderUpdate = serde_json::from_value(serde_json::json!({
            "slug": "gateway",
            "endpoint": "https://provider.example/v1",
            "tool_call_codec": wire
        }))
        .unwrap();
        assert_eq!(provider.tool_call_codec, mode);
    }
    let legacy: CloudProviderUpdate = serde_json::from_value(serde_json::json!({
        "slug": "gateway",
        "endpoint": "https://provider.example/v1"
    }))
    .unwrap();
    assert_eq!(legacy.tool_call_codec, ToolCallCodec::Native);
}

#[test]
fn rpc_model_transports_preserve_exact_model_ids_and_protocols() {
    use crate::config::schema::cloud_providers::{normalize_model_transports, ModelTransport};

    let input: CloudProviderUpdate = serde_json::from_value(serde_json::json!({
        "slug": "mixed-gateway",
        "endpoint": "https://example.net/v1",
        "model_transports": {
            "vendor/responses-v2": "responses",
            "legacy-chat": "chat_completions",
            "default": "auto"
        }
    }))
    .expect("valid wire transport selection");

    let routes = normalize_model_transports(input.model_transports).unwrap();
    assert_eq!(routes["vendor/responses-v2"], ModelTransport::Responses);
    assert_eq!(routes["legacy-chat"], ModelTransport::ChatCompletions);
    assert_eq!(routes["default"], ModelTransport::Auto);
}

#[test]
fn rpc_model_transports_reject_invalid_ids_before_persistence() {
    use crate::config::schema::cloud_providers::normalize_model_transports;

    let input: CloudProviderUpdate = serde_json::from_value(serde_json::json!({
        "slug": "mixed-gateway",
        "endpoint": "https://example.net/v1",
        "model_transports": {" padded-model ": "responses"}
    }))
    .expect("serde permits the key; normalization validates it");
    assert!(normalize_model_transports(input.model_transports).is_err());
}
