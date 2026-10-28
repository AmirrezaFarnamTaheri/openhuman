use super::*;

#[tokio::test]
async fn byok_multi_key_route_retries_pre_output_429_with_an_individual_credential() {
    use tinyinference_llm::message::Message;
    use tinyinference_llm::model::ModelRequest;
    use wiremock::matchers::header;

    let _guard = crate::inference::inference_test_guard_async().await;
    let _env = crate::config::TEST_ENV_LOCK.lock().await;
    let server = MockServer::start().await;
    let mut provider = openai_entry("p_rotation_test", "rotator-test");
    provider.endpoint = format!("{}/v1", server.uri());
    provider.default_model = Some("test-model".to_owned());

    let tmp = TempDir::new().expect("isolated credential workspace");
    let mut config = config_with_providers_in_tempdir(&tmp, vec![provider]);
    config.chat_provider = Some("rotator-test:test-model".to_owned());
    store_byo_key(&config, "rotator-test", "fleet-key-a;fleet-key-b");

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer fleet-key-a"))
        .respond_with(ResponseTemplate::new(429).set_body_json(serde_json::json!({
            "error": {"message": "rate limited", "type": "rate_limit_error", "code": "rate_limit_exceeded"}
        })))
        .expect(1)
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("authorization", "Bearer fleet-key-b"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chatcmpl-rotation-test",
            "object": "chat.completion",
            "created": 1770000000,
            "model": "test-model",
            "choices": [{
                "index": 0,
                "message": {"role": "assistant", "content": "ok"},
                "finish_reason": "stop"
            }],
            "usage": {"prompt_tokens": 1, "completion_tokens": 1, "total_tokens": 2}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let model = create_chat_model_with_model_id_in(
        &fixture_library_dispatch(), "chat", &config, 0.0,
    )
    .expect("create native provider model").0;
    let result = model
        .invoke(&(), ModelRequest::new(vec![Message::user("hello")]))
        .await;
    assert!(result.is_ok(), "429 should fail over before exposing output: {result:?}");
    server.verify().await;
}
