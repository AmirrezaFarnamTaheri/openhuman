use super::*;
use serde_json::json;
use tinyinference_llm::message::{AssistantMessage, MessageDelta};
use tinyinference_llm::model::{ModelRequest, ModelResponse, ModelStreamItem};
use tinyinference_llm::tool::ToolSchema;

fn qnml_adapter(inner: Arc<dyn ChatModel<()>>) -> TextToolCompatModel {
    TextToolCompatModel::new(inner, ToolCallCodec::Qnml)
}

fn declared() -> Vec<ToolSchema> {
    vec![
        ToolSchema::new(
            "shell",
            "Run an approved command",
            json!({"type":"object","properties":{"command":{"type":"string"}},"required":["command"],"additionalProperties":false}),
        ),
        ToolSchema::new(
            "read_file",
            "Read a permitted file",
            json!({"type":"object","properties":{"path":{"type":"string"},"line":{"type":"integer"}},"required":["path"],"additionalProperties":false}),
        ),
    ]
}

fn qnml_two_calls() -> &'static str {
    r#"<|QNML|tool_calls>
<|QNML|invoke name="shell"><|QNML|parameter name="command"><![CDATA[printf "a<b & c"]]></|QNML|parameter></|QNML|invoke>
<|QNML|invoke name="read_file"><|QNML|parameter name="path"><![CDATA[README.md]]></|QNML|parameter><|QNML|parameter name="line">3</|QNML|parameter></|QNML|invoke>
</|QNML|tool_calls>"#
}

#[test]
fn decodes_qwen_qnml_into_declared_typed_tool_calls() {
    let calls = parse_qnml_calls(qnml_two_calls(), &declared())
        .expect("valid QNML")
        .expect("envelope recognized");
    assert_eq!(calls.len(), 2);
    assert_eq!(calls[0].name, "shell");
    assert_eq!(calls[0].arguments, json!({"command": "printf \"a<b & c\""}));
    assert_eq!(calls[1].name, "read_file");
    assert_eq!(calls[1].arguments, json!({"path":"README.md","line":3}));
    assert_ne!(calls[0].id, calls[1].id);
}

#[test]
fn plain_text_and_prose_examples_never_become_executable_tools() {
    assert!(parse_qnml_calls("hello there", &declared()).unwrap().is_none());
    assert!(parse_qnml_calls(
        "Example: <|QNML|tool_calls><|QNML|invoke name=\"shell\"></|QNML|invoke></|QNML|tool_calls>",
        &declared(),
    ).unwrap().is_none());
    assert!(parse_qnml_calls(qnml_two_calls(), &[]).unwrap().is_none());
}

#[test]
fn rejects_unknown_tools_and_invalid_arguments_before_creating_calls() {
    let unknown = qnml_two_calls().replace("name=\"shell\"", "name=\"delete_database\"");
    assert!(parse_qnml_calls(&unknown, &declared()).is_err());
    let invalid = qnml_two_calls().replace("name=\"line\">3", "name=\"line\">\"three\"");
    assert!(parse_qnml_calls(&invalid, &declared()).is_err());
}

#[test]
fn malformed_or_ambiguous_markup_fails_closed() {
    for raw in [
        "<|QNML|tool_calls><|QNML|invoke name=\"shell\">",
        "<|QNML|tool_calls><|QNML|invoke name=\"shell\"></|QNML|invoke></|QNML|tool_calls>",
        "<|QNML|tool_calls><|QNML|invoke name=\"shell\"><|QNML|parameter name=\"command\">pwd</|QNML|parameter>",
        "<|QNML|tool_calls><|QNML|invoke name=\"shell\"><|QNML|parameter name=\"command\">pwd</|QNML|parameter><|QNML|parameter name=\"command\">ls</|QNML|parameter></|QNML|invoke></|QNML|tool_calls>",
        "<|QNML|tool_calls><|QNML|invoke name=\"shell\"><|QNML|parameter name=\"command\"><![CDATA[pwd]]></|QNML|parameter></|QNML|invoke>surprise</|QNML|tool_calls>",
    ] {
        assert!(parse_qnml_calls(raw, &declared()).is_err(), "accepted malformed: {raw}");
    }
}

#[test]
fn qnml_is_bounded_for_untrusted_web_gateway_output() {
    let oversized = format!(
        "<|QNML|tool_calls><|QNML|invoke name=\"shell\"><|QNML|parameter name=\"command\"><![CDATA[{}]]></|QNML|parameter></|QNML|invoke></|QNML|tool_calls>",
        "x".repeat(256 * 1024)
    );
    assert!(parse_qnml_calls(&oversized, &declared()).is_err());
    let mut repeated = String::from("<|QNML|tool_calls>");
    for _ in 0..17 {
        repeated.push_str("<|QNML|invoke name=\"read_file\"><|QNML|parameter name=\"path\">README.md</|QNML|parameter></|QNML|invoke>");
    }
    repeated.push_str("</|QNML|tool_calls>");
    assert!(parse_qnml_calls(&repeated, &declared()).is_err());
}

fn response(text: String) -> ModelResponse {
    ModelResponse {
        message: AssistantMessage {
            id: None,
            content: vec![ContentBlock::Text(text)],
            tool_calls: vec![],
            usage: None,
            origin: None,
        },
        usage: None,
        finish_reason: Some("stop".into()),
        raw: None,
        resolved_model: None,
        continue_turn: None,
        served_from_cache: false,
        correlation: None,
        resolved_route: None,
    }
}

#[test]
fn both_text_codecs_reject_undeclared_and_invalid_native_tool_calls() {
    for codec in [ToolCallCodec::Qnml, ToolCallCodec::TaggedJson] {
        let rewrite = |result| rewrite_text_codec_response(result, &declared(), codec);
        let mut undeclared = response("native".into());
        undeclared.message.tool_calls.push(ToolCall::new(
            "call-unknown", "delete_database", json!({}),
        ));
        assert!(rewrite(undeclared).is_err(), "{codec:?} accepted an undeclared native tool");

        let mut invalid_args = response("native".into());
        invalid_args.message.tool_calls.push(ToolCall::new(
            "call-invalid", "read_file", json!({"path": 17}),
        ));
        assert!(rewrite(invalid_args).is_err(), "{codec:?} accepted invalid native arguments");

        let mut valid = response("native".into());
        valid.message.tool_calls.push(ToolCall::new(
            "call-valid", "read_file", json!({"path": "README.md"}),
        ));
        assert_eq!(rewrite(valid.clone()).unwrap().message, valid.message);

        let mut duplicate = valid.clone();
        duplicate.message.tool_calls.push(ToolCall::new(
            "call-valid", "shell", json!({"command": "pwd"}),
        ));
        assert!(rewrite(duplicate).is_err(), "{codec:?} accepted duplicate call ids");

        let mut no_declarations = valid;
        assert!(
            rewrite_text_codec_response(no_declarations.clone(), &[], codec).is_err(),
            "{codec:?} accepted a native tool when tools were disabled"
        );
        no_declarations.message.tool_calls.clear();
        assert!(rewrite_text_codec_response(no_declarations, &[], codec).is_ok());
    }
}

#[test]
fn conversion_removes_raw_tagged_markup_but_preserves_native_tool_calls() {
    let converted = rewrite_qnml_response(response(qnml_two_calls().into()), &declared()).unwrap();
    assert!(converted.message.content.is_empty(), "tool markup must not leak into visible chat");
    assert_eq!(converted.message.tool_calls.len(), 2);
    assert_eq!(converted.finish_reason.as_deref(), Some("tool_calls"));

    let unchanged = response("normal assistant answer".into());
    let parsed = rewrite_qnml_response(unchanged.clone(), &declared()).unwrap();
    assert_eq!(parsed.message, unchanged.message);
    let mut native = response(qnml_two_calls().into());
    native.message.tool_calls.push(tinyinference_llm::tool::ToolCall::new(
        "native-call", "shell", json!({"command":"pwd"}),
    ));
    let preserved = rewrite_qnml_response(native.clone(), &declared()).unwrap();
    assert_eq!(preserved.message, native.message);
}

#[tokio::test]
async fn qnml_does_not_upgrade_markup_into_tools_when_tool_choice_is_none() {
    use futures::StreamExt;
    use tinyinference_llm::model::ToolChoice;

    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: qnml_two_calls().to_owned(),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::None,
        ..Default::default()
    };
    let response = adapter.invoke(&(), request.clone()).await.unwrap();
    assert!(
        response.message.tool_calls.is_empty(),
        "a declared tool must remain disabled when tool_choice is none"
    );

    let events: Vec<ModelStreamItem> = adapter
        .stream(&(), request)
        .await
        .unwrap()
        .collect()
        .await;
    assert!(
        events.iter().all(|event| !matches!(
            event,
            ModelStreamItem::Completed(result) if !result.message.tool_calls.is_empty()
        )),
        "the streaming path must also honor tool_choice none"
    );
}

#[tokio::test]
async fn qnml_cannot_call_a_different_declared_tool_than_explicitly_selected() {
    use futures::StreamExt;
    use tinyinference_llm::model::ToolChoice;

    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: qnml_two_calls().to_owned(),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::Tool("read_file".to_owned()),
        ..Default::default()
    };
    assert!(
        adapter.invoke(&(), request.clone()).await.is_err(),
        "shell is declared but not permitted by the named-tool policy"
    );
    let events: Vec<ModelStreamItem> = adapter
        .stream(&(), request)
        .await
        .unwrap()
        .collect()
        .await;
    assert!(
        events.iter().any(|event| matches!(event, ModelStreamItem::Failed(_))),
        "streaming must reject forbidden named-tool markup"
    );
}

#[tokio::test]
async fn explicit_no_tools_policy_never_executes_tagged_calls() {
    use tinyinference_llm::model::ToolChoice;
    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: qnml_two_calls().into(),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::None,
        ..Default::default()
    };
    let result = adapter.invoke(&(), request).await.unwrap();
    assert!(result.message.tool_calls.is_empty());
}

#[tokio::test]
async fn named_tool_choice_rejects_other_declared_tools() {
    use tinyinference_llm::model::ToolChoice;
    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: qnml_two_calls().into(),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::Tool("shell".into()),
        ..Default::default()
    };
    assert!(adapter.invoke(&(), request).await.is_err());
}

#[tokio::test]
async fn required_tool_policy_rejects_plain_text_from_qnml_gateway() {
    use futures::StreamExt;
    use tinyinference_llm::model::ToolChoice;

    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: "I cannot use tools right now".to_owned(),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::Required,
        ..Default::default()
    };
    assert!(
        adapter.invoke(&(), request.clone()).await.is_err(),
        "required tool output may not silently become a plain chat reply"
    );
    let events: Vec<ModelStreamItem> = adapter
        .stream(&(), request).await.unwrap().collect().await;
    assert!(
        events.iter().any(|event| matches!(event, ModelStreamItem::Failed(_))),
        "required tool streaming must fail if the gateway emits no tool"
    );
}

#[tokio::test]
async fn named_tool_policy_rejects_plain_text_without_the_selected_tool() {
    use futures::StreamExt;
    use tinyinference_llm::model::ToolChoice;

    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: "Finished without tool calls".into(),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::Tool("read_file".into()),
        ..Default::default()
    };
    assert!(adapter.invoke(&(), request.clone()).await.is_err());
    let events: Vec<ModelStreamItem> = adapter
        .stream(&(), request).await.unwrap().collect().await;
    assert!(events.iter().any(|event| matches!(event, ModelStreamItem::Failed(_))));
}

#[tokio::test]
async fn auto_choice_may_return_ordinary_text_without_tool_execution() {
    use tinyinference_llm::model::ToolChoice;

    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: "A normal answer".to_owned(),
    }));
    let response = adapter.invoke(&(), ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::Auto,
        ..Default::default()
    }).await.unwrap();
    assert!(response.message.tool_calls.is_empty());
    assert_eq!(response.text(), "A normal answer");
}


#[test]
fn tagged_json_parallel_and_single_tool_calls_share_native_validation() {
    let parallel = r#"<think>check the relevant files</think>
<tool_calls>[{"name":"read_file","arguments":{"path":"README.md","line":3}},{"name":"shell","arguments":{"command":"pwd"}}]</tool_calls>"#;
    let tagged = parse_tagged_json_output(parallel, &declared())
        .unwrap()
        .expect("tagged model output");
    assert_eq!(tagged.thinking.as_deref(), Some("check the relevant files"));
    assert_eq!(tagged.calls.len(), 2);
    assert_eq!(tagged.calls[0].arguments, json!({"path":"README.md","line":3}));
    assert_eq!(tagged.calls[1].arguments, json!({"command":"pwd"}));
    assert_ne!(tagged.calls[0].id, tagged.calls[1].id);

    let single = r#"<tool_call>{"name":"shell","arguments":{"command":"echo hello"}}</tool_call>"#;
    let tagged = parse_tagged_json_output(single, &declared()).unwrap().unwrap();
    assert_eq!(tagged.calls.len(), 1);
    assert_eq!(tagged.calls[0].name, "shell");
    assert_eq!(tagged.calls[0].arguments["command"], "echo hello");
}

#[test]
fn tagged_json_terminal_answers_strip_tags_and_keep_thinking_private() {
    let tagged_response = response("<think>reason privately</think><final_answer>Hello there</final_answer>".into());
    let converted = rewrite_tagged_json_response(tagged_response, &declared()).unwrap();
    assert_eq!(converted.message.content.len(), 2);
    assert!(matches!(
        converted.message.content[0],
        ContentBlock::Thinking { .. }
    ));
    assert_eq!(converted.message.content[1], ContentBlock::Text("Hello there".into()));
    assert_eq!(converted.finish_reason.as_deref(), Some("stop"));

    let ordinary = response("ordinary assistant response".into());
    let unchanged = rewrite_tagged_json_response(ordinary.clone(), &declared()).unwrap();
    assert_eq!(unchanged.message, ordinary.message);
}

#[test]
fn tagged_json_bounded_parser_rejects_undeclared_or_wrongly_typed_tool_calls() {
    for output in [
        r#"<tool_calls>[{"name":"delete_everything","arguments":{}}]</tool_calls>"#,
        r#"<tool_calls>[{"name":"read_file","arguments":{"path":4}}]</tool_calls>"#,
        r#"<tool_calls>[{"name":"read_file","arguments":"README.md"}]</tool_calls>"#,
        r#"<tool_calls>{"name":"read_file","arguments":{"path":"README.md"}}</tool_calls>"#,
        r#"<tool_calls>[]</tool_calls>"#,
        r#"<tool_calls>[{"name":"read_file","arguments":{"path":"README.md"}}]"#,
        r#"<tool_call>{"name":"read_file","arguments":{"path":"README.md"}}"#,
        r#"<think>reasoning</think>"#,
        r#"<final_answer>answer</final_answer><tool_calls>[{"name":"shell","arguments":{"command":"pwd"}}]</tool_calls>"#,
    ] {
        assert!(parse_tagged_json_output(output, &declared()).is_err(), "invalid tagged JSON accepted: {output}");
    }
    let too_many = format!(
        "<tool_calls>[{}]</tool_calls>",
        (0..17)
            .map(|_| r#"{"name":"shell","arguments":{"command":"pwd"}}"#)
            .collect::<Vec<_>>()
            .join(",")
    );
    assert!(parse_tagged_json_output(&too_many, &declared()).is_err());
    let oversized = format!("<final_answer>{}</final_answer>", "x".repeat(256 * 1024));
    assert!(parse_tagged_json_output(&oversized, &declared()).is_err());
}

#[test]
fn tagged_json_does_not_execute_tool_tag_examples_embedded_in_prose() {
    let prose = r#"For example: <tool_call>{"name":"shell","arguments":{"command":"pwd"}}</tool_call>"#;
    assert!(parse_tagged_json_output(prose, &declared()).unwrap().is_none());
    let parsed = parse_tagged_json_output(
        "<final_answer>The text '<tool_call>' is documentation.</final_answer>",
        &declared()
    ).unwrap().unwrap();
    assert!(parsed.calls.is_empty());
    assert_eq!(parsed.answer.as_deref(), Some("The text '<tool_call>' is documentation."));
}

#[tokio::test]
async fn tagged_json_tool_choice_and_streaming_use_native_contracts() {
    use futures::StreamExt;
    use tinyinference_llm::model::ToolChoice;

    let adapter = TextToolCompatModel::new(
        Arc::new(FakeTaggedModel {
            body: r#"<think>need file</think><tool_calls>[{"name":"read_file","arguments":{"path":"README.md"}}]</tool_calls>"#.into(),
        }),
        ToolCallCodec::TaggedJson,
    );
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::Tool("read_file".into()),
        ..Default::default()
    };
    let events: Vec<ModelStreamItem> = adapter.stream(&(), request).await.unwrap().collect().await;
    assert_eq!(events.len(), 2);
    let ModelStreamItem::Completed(response) = &events[1] else {
        panic!("expected typed completion, not text fragments");
    };
    assert_eq!(response.message.tool_calls.len(), 1);
    assert_eq!(response.message.tool_calls[0].name, "read_file");
    assert!(response.message.content.iter().any(|block| matches!(block, ContentBlock::Thinking { .. })));

    let forbidden = TextToolCompatModel::new(
        Arc::new(FakeTaggedModel {
            body: r#"<tool_call>{"name":"shell","arguments":{"command":"pwd"}}</tool_call>"#.into(),
        }),
        ToolCallCodec::TaggedJson,
    );
    let no_tools = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::None,
        ..Default::default()
    };
    assert!(forbidden.invoke(&(), no_tools).await.is_err());
}


#[test]
fn tagged_gateway_request_uses_text_tool_instructions_without_native_tool_wires() {
    use tinyinference_llm::message::Message;
    use tinyinference_llm::model::ToolChoice;
    let source = ModelRequest {
        messages: vec![Message::user("Read the repo README")],
        tools: declared(),
        tool_choice: ToolChoice::Tool("read_file".into()),
        prompt_fingerprint: Some("old-native-tool-prompt".into()),
        ..Default::default()
    };
    let adapted = prepare_compat_request(source.clone(), ToolCallCodec::TaggedJson).unwrap();
    assert!(adapted.tools.is_empty(), "upstream text gateway must not receive native tools");
    assert_eq!(adapted.tool_choice, ToolChoice::None);
    assert!(adapted.prompt_fingerprint.is_none(), "format change invalidates old cache prefix");
    assert_eq!(source.tools.len(), 2, "the caller's typed declarations must remain intact");
    let Some(Message::System(system)) = adapted.messages.first() else {
        panic!("tagged protocol instructions must be a system message");
    };
    let ContentBlock::Text(instructions) = &system.content[0] else {
        panic!("expected text protocol instructions");
    };
    assert!(instructions.contains("<tool_calls>"));
    assert!(instructions.contains("<final_answer>"));
    assert!(instructions.contains("read_file"));
    assert!(!instructions.contains("shell"), "named tool choice must not expose sibling tools");
}

#[test]
fn qnml_disabled_tool_choice_never_sends_native_tool_schemas_to_text_only_gateway() {
    use tinyinference_llm::model::ToolChoice;
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::None,
        ..Default::default()
    };
    let adapted = prepare_compat_request(request, ToolCallCodec::Qnml).unwrap();
    assert!(
        adapted.tools.is_empty(),
        "disabled tools must not appear on the native wire for text-only 2API gateways"
    );
    assert_eq!(adapted.tool_choice, ToolChoice::None);
}

#[test]
fn qnml_text_tools_use_prompt_instructions_without_native_function_call_payloads() {
    use tinyinference_llm::message::Message;
    let source = ModelRequest {
        messages: vec![Message::user("Work")],
        tools: declared(),
        ..Default::default()
    };
    let adapted = prepare_compat_request(source, ToolCallCodec::Qnml).unwrap();
    assert!(adapted.tools.is_empty());
    let Some(Message::System(system)) = adapted.messages.first() else {
        panic!("missing QNML protocol instruction");
    };
    let ContentBlock::Text(instructions) = &system.content[0] else {
        panic!("expected text instructions");
    };
    assert!(instructions.contains("<|QNML|tool_calls>"));
    assert!(instructions.contains("<|QNML|invoke"));
    assert!(instructions.contains("read_file"));
    assert!(instructions.contains("shell"));
}

#[test]
fn native_provider_requests_are_left_byte_for_byte_unchanged() {
    use tinyinference_llm::message::Message;
    let source = ModelRequest {
        messages: vec![Message::user("Plain text")],
        tools: declared(),
        ..Default::default()
    };
    let orig = serde_json::to_value(&source).unwrap();
    let adapted = prepare_compat_request(source, ToolCallCodec::Native).unwrap();
    assert_eq!(serde_json::to_value(adapted).unwrap(), orig);
}

struct FakeTaggedModel {
    body: String,
}

#[async_trait]
impl ChatModel<()> for FakeTaggedModel {
    async fn invoke(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelResponse> {
        Ok(response(self.body.clone()))
    }

    async fn stream(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelStream> {
        let text = self.body.clone();
        Ok(ModelStream::new(Box::pin(futures::stream::iter(vec![
            ModelStreamItem::Started,
            ModelStreamItem::MessageDelta(MessageDelta::text(text.clone())),
            ModelStreamItem::Completed(response(text)),
        ]))))
    }
}

#[tokio::test]
async fn qnml_stream_buffers_tool_markup_and_emits_one_normalized_terminal_result() {
    use futures::StreamExt;
    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: qnml_two_calls().to_string(),
    }));
    let request = ModelRequest {
        tools: declared(),
        ..Default::default()
    };
    let stream = adapter.stream(&(), request).await.unwrap();
    let events: Vec<ModelStreamItem> = stream.collect().await;
    assert_eq!(events.len(), 2, "must not expose partial tool tags as chat text");
    assert!(matches!(events[0], ModelStreamItem::Started));
    let ModelStreamItem::Completed(result) = &events[1] else {
        panic!("expected a successful terminal result");
    };
    assert_eq!(result.message.tool_calls.len(), 2);
    assert!(result.message.content.is_empty());
}


#[tokio::test]
async fn tagged_json_final_answer_never_flashes_protocol_tags_in_chat() {
    use futures::StreamExt;
    let adapter = TextToolCompatModel::new(
        Arc::new(FakeTaggedModel {
            body: "<think>private</think><final_answer>Safe, clean answer</final_answer>".into(),
        }),
        ToolCallCodec::TaggedJson,
    );
    let events: Vec<ModelStreamItem> = adapter
        .stream(&(), ModelRequest::default())
        .await
        .unwrap()
        .collect()
        .await;
    assert_eq!(events.len(), 2);
    let ModelStreamItem::Completed(response) = &events[1] else {
        panic!("expected terminal normalized answer");
    };
    assert_eq!(
        response.message.content.last(),
        Some(&ContentBlock::Text("Safe, clean answer".into())),
    );
    assert!(!response.message.content.iter().any(|block| {
        matches!(block, ContentBlock::Text(text) if text.contains("<final_answer>"))
    }));
}

struct RecordingTextGateway {
    received: Arc<std::sync::Mutex<Option<ModelRequest>>>,
}

#[async_trait]
impl ChatModel<()> for RecordingTextGateway {
    async fn invoke(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelResponse> {
        Ok(response("unused".into()))
    }

    async fn stream(&self, _: &(), request: ModelRequest) -> tinyinference_llm::Result<ModelStream> {
        *self.received.lock().unwrap() = Some(request);
        Ok(ModelStream::new(Box::pin(futures::stream::iter(vec![
            ModelStreamItem::Started,
            ModelStreamItem::MessageDelta(MessageDelta::text("hello".into())),
            ModelStreamItem::Completed(response("hello".into())),
        ]))))
    }
}

#[tokio::test]
async fn qnml_stream_disables_wire_tool_schemas_when_tool_choice_none() {
    use futures::StreamExt;
    let received = Arc::new(std::sync::Mutex::new(None));
    let adapter = qnml_adapter(Arc::new(RecordingTextGateway {
        received: Arc::clone(&received),
    }));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::None,
        ..Default::default()
    };
    let events: Vec<ModelStreamItem> = adapter
        .stream(&(), request)
        .await
        .unwrap()
        .collect()
        .await;
    assert!(events.iter().any(|event| matches!(event, ModelStreamItem::Completed(_))));
    let guard = received.lock().unwrap();
    let wire = guard.as_ref().expect("gateway must receive a request");
    assert!(wire.tools.is_empty(), "native tool declarations must not be sent");
    assert_eq!(wire.tool_choice, ToolChoice::None);
}

#[tokio::test]
async fn ordinary_chat_stream_still_emits_native_text_deltas() {
    use futures::StreamExt;
    let adapter = qnml_adapter(Arc::new(FakeTaggedModel {
        body: "ordinary answer".into(),
    }));
    let stream = adapter.stream(&(), ModelRequest::default()).await.unwrap();
    let events: Vec<ModelStreamItem> = stream.collect().await;
    assert_eq!(events.len(), 3);
    assert!(matches!(events[1], ModelStreamItem::MessageDelta(_)));
}
