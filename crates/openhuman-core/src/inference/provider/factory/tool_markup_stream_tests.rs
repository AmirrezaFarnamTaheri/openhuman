use super::*;

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
    assert_eq!(
        events.len(),
        2,
        "must not expose partial tool tags as chat text"
    );
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

    async fn stream(
        &self,
        _: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelStream> {
        *self.received.lock().unwrap() = Some(request);
        Ok(ModelStream::new(Box::pin(futures::stream::iter(vec![
            ModelStreamItem::Started,
            ModelStreamItem::MessageDelta(MessageDelta::text("hello")),
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
    let events: Vec<ModelStreamItem> = adapter.stream(&(), request).await.unwrap().collect().await;
    assert!(events
        .iter()
        .any(|event| matches!(event, ModelStreamItem::Completed(_))));
    let guard = received.lock().unwrap();
    let wire = guard.as_ref().expect("gateway must receive a request");
    assert!(
        wire.tools.is_empty(),
        "native tool declarations must not be sent"
    );
    assert_eq!(wire.tool_choice, ToolChoice::None);
}

struct UnexpectedNativeToolStream;

#[async_trait]
impl ChatModel<()> for UnexpectedNativeToolStream {
    async fn invoke(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelResponse> {
        Ok(response("unused".into()))
    }

    async fn stream(&self, _: &(), _: ModelRequest) -> tinyinference_llm::Result<ModelStream> {
        let mut result = response("not allowed".into());
        result.message.tool_calls.push(ToolCall::new(
            "attempt-1",
            "shell",
            json!({"command":"echo must-not-run"}),
        ));
        Ok(ModelStream::new(Box::pin(futures::stream::iter(vec![
            ModelStreamItem::Started,
            ModelStreamItem::Completed(result),
        ]))))
    }
}

#[tokio::test]
async fn qnml_no_tools_stream_fails_closed_on_unrequested_native_calls() {
    use futures::StreamExt;
    let adapter = qnml_adapter(Arc::new(UnexpectedNativeToolStream));
    let request = ModelRequest {
        tools: declared(),
        tool_choice: ToolChoice::None,
        ..Default::default()
    };
    let events: Vec<ModelStreamItem> = adapter.stream(&(), request).await.unwrap().collect().await;
    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], ModelStreamItem::Started));
    assert!(matches!(events[1], ModelStreamItem::Failed(_)));
    assert!(!events
        .iter()
        .any(|event| matches!(event, ModelStreamItem::Completed(_))));
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
