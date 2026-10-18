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
