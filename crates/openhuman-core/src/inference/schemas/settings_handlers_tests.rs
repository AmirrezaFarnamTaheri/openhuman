use super::*;
use crate::config::schema::cloud_providers::ToolCallCodec;

#[test]
fn rpc_tool_codec_field_keeps_json_tagged_or_qnml_selection() {
    for (wire, expected) in [
        ("native", ToolCallCodec::Native),
        ("qnml", ToolCallCodec::Qnml),
        ("tagged_json", ToolCallCodec::TaggedJson),
    ] {
        let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
            "slug": "peer-gateway",
            "endpoint": "https://example.net/v1",
            "tool_call_codec": wire,
        }))
        .expect("valid codec name");
        assert_eq!(input.tool_call_codec, expected);
    }
}

#[test]
fn older_provider_update_payload_defaults_to_native_tool_calls() {
    let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
        "slug": "legacy",
        "endpoint": "https://example.net/v1"
    }))
    .unwrap();
    assert_eq!(input.tool_call_codec, ToolCallCodec::Native);
    assert!(input.model_transports.is_empty());
}

#[test]
fn rpc_model_transports_preserve_exact_model_ids_and_protocols() {
    use crate::config::schema::cloud_providers::{normalize_model_transports, ModelTransport};

    let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
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

    let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
        "slug": "mixed-gateway",
        "endpoint": "https://example.net/v1",
        "model_transports": {" padded-model ": "responses"}
    }))
    .expect("serde permits the key; normalization validates it");
    assert!(normalize_model_transports(input.model_transports).is_err());
}
