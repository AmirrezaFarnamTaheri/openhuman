//! The context-compaction summarizer's model: `[context] summarizer_model`
//! names a model on the turn's own provider route, and an unset (or blank)
//! value keeps the turn's model.

use std::sync::Arc;

use tinyinference_llm::message::Message;
use tinyinference_llm::model::ModelRequest;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

use crate::agent::tinyagents::TurnModelSource;
use crate::config::schema::cloud_providers::{AuthStyle, CloudProviderCreds};

const TURN_MODEL: &str = "fixture/turn-model";
const SUMMARIZER_MODEL: &str = "scaledown/summarize";

/// The models one turn's summarizer and primary put on the wire, and the
/// summarizer model id the turn reports for compaction provenance.
struct WireModels {
    summarizer: String,
    primary: String,
    summarizer_model_id: String,
}

/// Build a BYOK OpenRouter turn against a stand-in endpoint, call its
/// summarizer then its primary once each, and read back the `model` field of
/// each request.
async fn wire_models(summarizer_model: Option<&str>) -> WireModels {
    let _guard = crate::inference::inference_test_guard_async().await;
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-fixture",
            "object": "chat.completion",
            "created": 0,
            "model": "fixture",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "ok"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        })))
        .mount(&server)
        .await;

    let tmp = tempfile::tempdir().expect("isolated workspace");
    let mut config = crate::config::Config {
        config_path: tmp.path().join("config.toml"),
        workspace_dir: tmp.path().join("workspace"),
        ..Default::default()
    };
    config.cloud_providers.push(CloudProviderCreds {
        id: "p_openrouter".into(),
        slug: "openrouter".into(),
        label: "OpenRouter".into(),
        endpoint: format!("{}/api/v1", server.uri()),
        auth_style: AuthStyle::Bearer,
        default_model: Some(TURN_MODEL.into()),
    });
    config.chat_provider = Some(format!("openrouter:{TURN_MODEL}"));
    config.context.summarizer_model = summarizer_model.map(str::to_owned);
    crate::security::credentials::AuthService::from_config(&config)
        .store_provider_token(
            "provider:openrouter",
            "default",
            "sk-fixture-openrouter",
            Default::default(),
            true,
        )
        .expect("store isolated provider credential");

    let dispatch = crate::core::runtime::DispatchCtx::new(
        crate::core::runtime::CoreContext::for_test_with_host_kind(
            crate::core::types::HostKind::Library,
            crate::core::runtime::DomainSet::full(),
            None,
        ),
    );
    let models = TurnModelSource::new_crate_native(&dispatch, "chat", Arc::new(config))
        .build(TURN_MODEL, 0.0, None, None)
        .expect("a BYOK cloud turn builds");
    let request = || ModelRequest::new(vec![Message::user("condense this")]);
    models
        .summarizer
        .invoke(&(), request())
        .await
        .expect("the summarizer reaches the stand-in endpoint");
    models
        .primary
        .invoke(&(), request())
        .await
        .expect("the primary reaches the stand-in endpoint");

    let requests = server
        .received_requests()
        .await
        .expect("the stand-in endpoint records requests");
    let sent: Vec<String> = requests
        .iter()
        .map(|request| {
            let body: serde_json::Value =
                serde_json::from_slice(&request.body).expect("a JSON request body");
            body["model"].as_str().unwrap_or_default().to_owned()
        })
        .collect();
    assert_eq!(
        sent.len(),
        2,
        "one summarizer and one primary call: {sent:?}"
    );
    WireModels {
        summarizer: sent[0].clone(),
        primary: sent[1].clone(),
        summarizer_model_id: models.summarizer_model_id().to_owned(),
    }
}

#[tokio::test]
async fn a_configured_summarizer_model_is_called_on_the_turns_route() {
    let models = wire_models(Some(SUMMARIZER_MODEL)).await;

    assert_eq!(models.summarizer, SUMMARIZER_MODEL);
    assert_eq!(models.summarizer_model_id, SUMMARIZER_MODEL);
    assert_eq!(models.primary, TURN_MODEL, "the turn keeps its own model");
}

#[tokio::test]
async fn without_a_configured_summarizer_model_the_summarizer_uses_the_turn_model() {
    let models = wire_models(None).await;

    assert_eq!(models.summarizer, TURN_MODEL);
    assert_eq!(models.summarizer_model_id, TURN_MODEL);
    assert_eq!(models.primary, TURN_MODEL);
}

#[tokio::test]
async fn a_blank_summarizer_model_counts_as_unset() {
    let models = wire_models(Some("  ")).await;

    assert_eq!(models.summarizer, TURN_MODEL);
    assert_eq!(models.summarizer_model_id, TURN_MODEL);
}
