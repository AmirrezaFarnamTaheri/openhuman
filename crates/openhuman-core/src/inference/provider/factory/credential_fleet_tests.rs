use super::*;
use tinyinference_llm::message::{ImageRef, UserMessage};
#[test]
fn image_endpoints_only_apply_when_request_contains_images() {
    let text = ModelRequest {
        messages: vec![Message::user("text")],
        ..Default::default()
    };
    assert!(!request_has_image(&text));
    let image = ModelRequest {
        messages: vec![Message::User(UserMessage {
            content: vec![
                ContentBlock::Text("What's here?".into()),
                ContentBlock::Image(ImageRef {
                    url: "data:image/png;base64,AAAA".into(),
                    mime_type: Some("image/png".into()),
                }),
            ],
        })],
        ..Default::default()
    };
    assert!(request_has_image(&image));
}
struct StubStreamModel {
    fail_auth: bool,
}
#[async_trait]
impl ChatModel<()> for StubStreamModel {
    async fn invoke(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelResponse> {
        Err(tinyinference_llm::Error::Model(
            "not used by stream test".into(),
        ))
    }
    async fn stream(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelStream> {
        let events = if self.fail_auth {
            vec![
                ModelStreamItem::Started,
                ModelStreamItem::ProviderFailed(ProviderError {
                    status: Some(429),
                    ..Default::default()
                }),
            ]
        } else {
            vec![
                ModelStreamItem::Started,
                ModelStreamItem::MessageDelta(tinyinference_llm::message::MessageDelta {
                    text: "selected-second-key".into(),
                    reasoning: String::new(),
                    tool_call: None,
                }),
                ModelStreamItem::Failed("test stream finished".into()),
            ]
        };
        Ok(ModelStream::new(Box::pin(futures::stream::iter(events))))
    }
}
#[tokio::test]
async fn pre_content_rate_limit_rotates_stream_before_exposing_error() {
    use futures::StreamExt;
    let scope = "fleet-stream-first-429";
    let fleet = CredentialFleetModel::new(
        scope.into(),
        vec!["first".into(), "second".into()],
        vec![
            Arc::new(StubStreamModel { fail_auth: true }),
            Arc::new(StubStreamModel { fail_auth: false }),
        ],
        None,
        RotationStrategy::RoundRobin,
    );
    let stream = fleet.stream(&(), ModelRequest::default()).await.unwrap();
    let events: Vec<ModelStreamItem> = stream.collect().await;
    assert!(matches!(
        events.get(1),
        Some(ModelStreamItem::MessageDelta(_))
    ));
    assert_eq!(key_pool::snapshot(scope)[0].status, "cooldown");
}

#[test]
fn deferred_route_is_keyed_to_credential_not_array_position() {
    let mut handle = tinyinference_llm::model::DeferredHandle::new("openai", "resp_123");
    stamp_deferred_handle(&mut handle, "secret-one", true);
    assert!(serde_json::to_string(&handle)
        .unwrap()
        .find("secret-one")
        .is_none());
    let route = decode_deferred_handle(&handle).unwrap();
    assert_eq!(route.key_fingerprint, credential_fingerprint("secret-one"));
    assert!(route.image);
}

#[test]
fn provider_failures_are_rerouted_conservatively() {
    assert_eq!(
        classify(Some(401), None, None).unwrap().failure,
        KeyFailure::Authentication
    );
    assert_eq!(
        classify(Some(429), None, Some(5000)).unwrap().failure,
        KeyFailure::RateLimited
    );
    assert_eq!(
        classify(Some(403), Some("quota_exhausted"), None)
            .unwrap()
            .failure,
        KeyFailure::RateLimited
    );
    assert!(classify(Some(400), None, None).is_none());
    assert!(classify(Some(403), Some("forbidden"), None).is_none());
}
