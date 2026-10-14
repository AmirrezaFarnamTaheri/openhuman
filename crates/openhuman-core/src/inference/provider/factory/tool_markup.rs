//! Opt-in 2API/Qwen QNML text-tool adapter for native ChatModel requests.
//! Only declared tools are converted, and no text-only provider gains permission
//! to call undeclared tools. Tool-enabled streams buffer until terminal output.

use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::{Map, Value};
use tinyinference_llm::message::{ContentBlock, Message};
use tinyinference_llm::model::{
    ChatModel, DeferredHandle, DeferredStatus, InputModality, InputSource, ModelProfile,
    ModelRequest, ModelResponse, ModelStream, ModelStreamItem, StreamAccumulator, ToolChoice,
};
use tinyinference_llm::tool::{ToolCall, ToolSchema};
use crate::config::schema::cloud_providers::ToolCallCodec;

#[path = "tool_tagged_json.rs"]
mod tagged_json;
#[cfg(test)]
use tagged_json::parse_tagged_json_output;
use tagged_json::rewrite_tagged_json_response;

const OPEN_CALLS: &str = "<|QNML|tool_calls>";
const CLOSE_CALLS: &str = "</|QNML|tool_calls>";
const OPEN_INVOKE: &str = "<|QNML|invoke name=\"";
const CLOSE_INVOKE: &str = "</|QNML|invoke>";
const OPEN_PARAM: &str = "<|QNML|parameter name=\"";
const CLOSE_PARAM: &str = "</|QNML|parameter>";
const MAX_OUTPUT_BYTES: usize = 256 * 1024;
const MAX_TOOL_CALLS: usize = 16;
const MAX_ARGUMENTS_PER_CALL: usize = 48;
const MAX_VALUE_BYTES: usize = 64 * 1024;
static NEXT_QNML_CALL: AtomicU64 = AtomicU64::new(1);

fn invalid(reason: &'static str) -> tinyinference_llm::Error {
    tinyinference_llm::Error::Validation(format!("QNML tool-call response: {reason}"))
}

fn name_and_rest<'a>(
    raw: &'a str,
    open: &str,
) -> tinyinference_llm::Result<(&'a str, &'a str)> {
    let rest = raw.strip_prefix(open).ok_or_else(|| invalid("expected opening tag"))?;
    let end = rest.find("\">").ok_or_else(|| invalid("unterminated name attribute"))?;
    let name = &rest[..end];
    if name.is_empty()
        || name.len() > 128
        || !name.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        return Err(invalid("invalid tool or parameter name"));
    }
    Ok((name, &rest[end + 2..]))
}

fn parameter_value(raw: &str) -> tinyinference_llm::Result<Value> {
    let value = if raw.starts_with("<![CDATA[") {
        raw.strip_prefix("<![CDATA[")
            .and_then(|text| text.strip_suffix("]]>"))
            .ok_or_else(|| invalid("unterminated CDATA parameter"))?
    } else {
        if raw.contains("<|QNML|") || raw.contains("</|QNML|") || raw.contains("<![CDATA[") {
            return Err(invalid("nested markup inside parameter"));
        }
        raw.trim()
    };
    if value.len() > MAX_VALUE_BYTES {
        return Err(invalid("parameter value exceeds size limit"));
    }
    Ok(serde_json::from_str(value).unwrap_or_else(|_| Value::String(value.to_owned())))
}

/// Parses only a complete top-level tagged QNML envelope. Ordinary prose
/// remains prose even if it quotes tool syntax in an example or code block.
pub(super) fn parse_qnml_calls(
    output: &str,
    declared: &[ToolSchema],
) -> tinyinference_llm::Result<Option<Vec<ToolCall>>> {
    if declared.is_empty() {
        return Ok(None);
    }
    let trimmed = output.trim();
    if !trimmed.starts_with(OPEN_CALLS) {
        return Ok(None);
    }
    if trimmed.len() > MAX_OUTPUT_BYTES {
        return Err(invalid("response exceeds tool-call size limit"));
    }
    let body = trimmed
        .strip_prefix(OPEN_CALLS)
        .and_then(|value| value.strip_suffix(CLOSE_CALLS))
        .ok_or_else(|| invalid("missing closing tool_calls tag"))?;
    let mut remaining = body.trim();
    let mut calls = Vec::new();
    while !remaining.is_empty() {
        if calls.len() >= MAX_TOOL_CALLS {
            return Err(invalid("too many tool calls"));
        }
        let (name, after_open) = name_and_rest(remaining, OPEN_INVOKE)?;
        let declared_tool = declared
            .iter()
            .find(|tool| tool.name == name)
            .ok_or_else(|| invalid("tool not declared by request"))?;
        let (parameters, rest) = after_open
            .split_once(CLOSE_INVOKE)
            .ok_or_else(|| invalid("missing closing invoke tag"))?;
        remaining = rest.trim_start();
        let mut parameter_text = parameters.trim();
        let mut arguments = Map::new();
        while !parameter_text.is_empty() {
            if arguments.len() >= MAX_ARGUMENTS_PER_CALL {
                return Err(invalid("too many tool parameters"));
            }
            let (parameter, body) = name_and_rest(parameter_text, OPEN_PARAM)?;
            let (value, rest) = body
                .split_once(CLOSE_PARAM)
                .ok_or_else(|| invalid("missing closing parameter tag"))?;
            if arguments
                .insert(parameter.to_owned(), parameter_value(value)?)
                .is_some()
            {
                return Err(invalid("duplicate tool parameter"));
            }
            parameter_text = rest.trim_start();
        }
        let sequence = NEXT_QNML_CALL.fetch_add(1, Ordering::Relaxed);
        let call = ToolCall::new(
            format!("call_qnml_{:x}_{sequence:x}", std::process::id()),
            name,
            Value::Object(arguments),
        );
        declared_tool.validate_call(&call)?;
        calls.push(call);
    }
    if calls.is_empty() {
        return Err(invalid("tool_calls envelope contains no invocations"));
    }
    let mut ids = HashSet::new();
    debug_assert!(calls.iter().all(|call| ids.insert(&call.id)));
    Ok(Some(calls))
}

/// Preserve the caller's executable-tool restrictions even when the upstream
/// transport serializes calls as plain markup rather than native tool deltas.
/// Model-provided QNML must never expand ToolChoice::None or ToolChoice::Tool.
fn allowed_qnml_tools(request: &ModelRequest) -> Vec<ToolSchema> {
    match &request.tool_choice {
        ToolChoice::None => Vec::new(),
        ToolChoice::Tool(name) => request
            .tools
            .iter()
            .filter(|tool| tool.name == *name)
            .cloned()
            .collect(),
        ToolChoice::Auto | ToolChoice::Required => request.tools.clone(),
    }
}

const MAX_TOOL_PROMPT_BYTES: usize = 64 * 1024;

/// Text-based 2API gateways may reject native OpenAI tool definitions entirely.
/// The explicit compatibility choice supplies the protocol contract as model
/// instructions and removes the native function-call request fields. The
/// *original* per-request declarations still gate every decoded call later.
fn prepare_compat_request(
    mut request: ModelRequest,
    codec: ToolCallCodec,
) -> tinyinference_llm::Result<ModelRequest> {
    if codec == ToolCallCodec::Native {
        return Ok(request);
    }
    let declared = allowed_qnml_tools(&request);
    if declared.is_empty() {
        if matches!(request.tool_choice, ToolChoice::Required | ToolChoice::Tool(_)) {
            return Err(invalid("selected or required tool was not declared"));
        }
        if codec == ToolCallCodec::Qnml {
            // Even when the user disables tool execution, a text-only gateway
            // may reject the native tools array. Preserve the no-tool policy
            // while removing unused native declarations from the wire.
            request.tools.clear();
            request.tool_choice = ToolChoice::None;
            return Ok(request);
        }
    }
    let tools_json = serde_json::to_string(&declared)
        .map_err(|_| invalid("unable to serialize declared tools"))?;
    if tools_json.len() > MAX_TOOL_PROMPT_BYTES {
        return Err(invalid("declared tool schema exceeds protocol prompt limit"));
    }
    let require_tool = matches!(request.tool_choice, ToolChoice::Required | ToolChoice::Tool(_));
    let instruction = match codec {
        ToolCallCodec::Qnml => format!(
            r#"2API QNML tool transport. Use only the declared tools below.
For every tool invocation emit this EXACT tagged grammar:
<|QNML|tool_calls>
<|QNML|invoke name="EXACT_DECLARED_NAME">
<|QNML|parameter name="PARAMETER"><![CDATA[value]]></|QNML|parameter>
</|QNML|invoke>
</|QNML|tool_calls>
Emit ordinary assistant text if no tool is needed. Never invent tool names.
Requirement: {}.
Declared tool schemas (JSON): {}"#,
            if require_tool {
                "at least one call required"
            } else {
                "call only when needed"
            },
            tools_json,
        ),
        ToolCallCodec::TaggedJson => format!(
            r#"2API tagged-JSON tool transport. Output at most one terminal block.
For tools emit <tool_calls>[{{"name":"EXACT_DECLARED_NAME","arguments":{{}}}}]</tool_calls>.
Otherwise emit <final_answer>your answer</final_answer>.
Optional <think>reasoning</think> blocks may precede the terminal block.
Use valid JSON with only declared tools and schema-validated arguments.
Do not output text before, after or between these terminal blocks.
Requirement: {}.
Declared tool schemas (JSON): {}"#,
            if require_tool {
                "at least one call required"
            } else if matches!(request.tool_choice, ToolChoice::None) {
                "tools are disabled"
            } else {
                "call only when needed"
            },
            tools_json,
        ),
        ToolCallCodec::Native => unreachable!("handled above"),
    };
    request.messages.insert(0, Message::system(instruction));
    request.tools.clear();
    request.tool_choice = ToolChoice::None;
    // The message list has changed, so previously computed stable-prefix
    // cache segments must not claim the old native-tool transcript identity.
    request.prompt_fingerprint = None;
    request.cache_segments.clear();
    Ok(request)
}

/// A provider's accepted wire format is not permission to silently weaken
/// the tool-choice contract. In particular, Required and Tool(name) need at
/// least one actual tool call, even when the gateway returns ordinary prose.
fn enforce_tool_choice(
    response: ModelResponse,
    choice: &ToolChoice,
) -> tinyinference_llm::Result<ModelResponse> {
    match choice {
        ToolChoice::Required if response.message.tool_calls.is_empty() => {
            Err(invalid("required tool call missing from model response"))
        }
        ToolChoice::Tool(_) if response.message.tool_calls.is_empty() => {
            Err(invalid("selected tool call missing from model response"))
        }
        ToolChoice::Tool(name)
            if response.message.tool_calls.iter().any(|call| call.name != *name) =>
        {
            Err(invalid("gateway returned a tool outside the named tool choice"))
        }
        ToolChoice::None if !response.message.tool_calls.is_empty() => {
            Err(invalid("gateway returned a tool despite tool_choice none"))
        }
        _ => Ok(response),
    }
}

/// A gateway may produce native tool_calls even when using an explicit
/// text-tool codec. Native does not mean trusted: these are untrusted model
/// output and must obey the exact same request declaration/schema boundary as
/// decoded QNML or tagged JSON. Reject, never silently discard, invalid calls.
fn validate_native_tool_calls(
    calls: &[ToolCall],
    declared: &[ToolSchema],
) -> tinyinference_llm::Result<()> {
    if calls.len() > MAX_TOOL_CALLS {
        return Err(invalid("too many native tool calls"));
    }
    let mut ids = HashSet::new();
    for call in calls {
        if call.id.is_empty() || !ids.insert(call.id.as_str()) {
            return Err(invalid("empty or duplicate native tool-call id"));
        }
        let schema = declared
            .iter()
            .find(|tool| tool.name == call.name)
            .ok_or_else(|| invalid("native tool not declared by request"))?;
        schema.validate_call(call)?;
    }
    Ok(())
}

/// Honors native tool_calls if already present: compatibility never overrides
/// a proper provider wire response, and never changes user-visible plain text.
pub(super) fn rewrite_qnml_response(
    mut response: ModelResponse,
    declared: &[ToolSchema],
) -> tinyinference_llm::Result<ModelResponse> {
    if !response.message.tool_calls.is_empty() {
        validate_native_tool_calls(&response.message.tool_calls, declared)?;
        return Ok(response);
    }
    if declared.is_empty() {
        return Ok(response);
    }
    let text_indices: Vec<_> = response
        .message
        .content
        .iter()
        .enumerate()
        .filter(|(_, block)| matches!(block, ContentBlock::Text(_)))
        .map(|(index, _)| index)
        .collect();
    if text_indices.len() != 1 {
        return Ok(response);
    }
    let index = text_indices[0];
    let ContentBlock::Text(ref text) = response.message.content[index] else {
        return Ok(response);
    };
    let Some(calls) = parse_qnml_calls(text, declared)? else {
        return Ok(response);
    };
    response.message.content.remove(index);
    response.message.tool_calls = calls;
    response.finish_reason = Some("tool_calls".into());
    Ok(response)
}

/// This adapter is selected by a per-provider explicit compatibility setting.
/// Credentials, transport, TLS, retries, approval, and execution stay native.
pub(super) struct TextToolCompatModel {
    inner: Arc<dyn ChatModel<()>>,
    codec: ToolCallCodec,
}

impl TextToolCompatModel {
    pub(super) fn new(inner: Arc<dyn ChatModel<()>>, codec: ToolCallCodec) -> Self {
        Self { inner, codec }
    }
}

fn rewrite_text_codec_response(
    response: ModelResponse,
    declared: &[ToolSchema],
    codec: ToolCallCodec,
) -> tinyinference_llm::Result<ModelResponse> {
    match codec {
        ToolCallCodec::Native => Ok(response),
        ToolCallCodec::Qnml => rewrite_qnml_response(response, declared),
        ToolCallCodec::TaggedJson => rewrite_tagged_json_response(response, declared),
    }
}

#[async_trait]
impl ChatModel<()> for TextToolCompatModel {
    fn profile(&self) -> Option<&ModelProfile> {
        self.inner.profile()
    }

    fn supports_input(&self, modality: InputModality, mime: &str, source: InputSource) -> bool {
        self.inner.supports_input(modality, mime, source)
    }

    fn cache_identity(&self) -> Option<String> {
        self.inner
            .cache_identity()
            .map(|identity| format!("{identity}:text-tool-{:?}-v1", self.codec))
    }

    async fn invoke(
        &self,
        state: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelResponse> {
        let declared = allowed_qnml_tools(&request);
        let choice = request.tool_choice.clone();
        let prepared = prepare_compat_request(request, self.codec)?;
        let response = self.inner.invoke(state, prepared).await?;
        enforce_tool_choice(rewrite_text_codec_response(response, &declared, self.codec)?, &choice)
    }

    async fn stream(
        &self,
        state: &(),
        request: ModelRequest,
    ) -> tinyinference_llm::Result<ModelStream> {
        let declared = allowed_qnml_tools(&request);
        let choice = request.tool_choice.clone();
        if declared.is_empty() && matches!(choice, ToolChoice::Required | ToolChoice::Tool(_)) {
            return Err(invalid("requested tool is not declared"));
        }
        // QNML has no text-only terminal grammar; keep streaming those turns.
        // Tagged JSON has <final_answer> blocks even without tools, so it
        // buffers that explicit provider mode to avoid exposing protocol tags.
        let codec = self.codec;
        let prepared = prepare_compat_request(request, self.codec)?;
        if declared.is_empty() && codec == ToolCallCodec::Qnml {
            return self.inner.stream(state, prepared).await;
        }
        let mut original = self.inner.stream(state, prepared).await?;
        let metadata = original.metadata().clone();
        let terminal = async move {
            let mut accumulator = StreamAccumulator::new();
            let mut bytes = 0usize;
            let mut completed = false;
            while let Some(event) = original.next().await {
                match &event {
                    ModelStreamItem::MessageDelta(delta) => {
                        bytes = bytes.saturating_add(delta.text.len() + delta.reasoning.len());
                    }
                    ModelStreamItem::ToolCallDelta(delta) => {
                        bytes = bytes.saturating_add(delta.content.len());
                    }
                    ModelStreamItem::Completed(response) => {
                        let response_bytes: usize = response
                            .message
                            .content
                            .iter()
                            .filter_map(|block| match block {
                                ContentBlock::Text(text) => Some(text.len()),
                                _ => None,
                            })
                            .sum();
                        // The final response repeats the content already seen
                        // in deltas, so count the larger view, not their sum.
                        bytes = bytes.max(response_bytes);
                        completed = true;
                    }
                    ModelStreamItem::Deferred(handle) => {
                        return ModelStreamItem::Deferred(handle.clone());
                    }
                    ModelStreamItem::ProviderFailed(error) => {
                        return ModelStreamItem::ProviderFailed(error.clone());
                    }
                    ModelStreamItem::Failed(reason) => {
                        return ModelStreamItem::Failed(reason.clone());
                    }
                    _ => {}
                }
                if bytes > MAX_OUTPUT_BYTES {
                    return ModelStreamItem::Failed(
                        "QNML compatibility output exceeded 256 KiB".into(),
                    );
                }
                accumulator.push(&event);
                if completed {
                    break;
                }
            }
            if !completed {
                return ModelStreamItem::Failed(
                    "QNML compatibility stream ended without completion".into(),
                );
            }
            match accumulator
                .finish()
                .and_then(|result| rewrite_text_codec_response(result, &declared, codec))
                .and_then(|result| enforce_tool_choice(result, &choice))
            {
                Ok(response) => ModelStreamItem::Completed(response),
                Err(tinyinference_llm::Error::Provider(error)) => {
                    ModelStreamItem::ProviderFailed(*error)
                }
                Err(error) => ModelStreamItem::Failed(error.to_string()),
            }
        };
        Ok(ModelStream::new(Box::pin(
            futures::stream::iter([ModelStreamItem::Started])
                .chain(futures::stream::once(terminal)),
        ))
        .with_metadata(metadata))
    }

    async fn fetch_deferred(
        &self,
        handle: &DeferredHandle,
    ) -> tinyinference_llm::Result<DeferredStatus> {
        self.inner.fetch_deferred(handle).await
    }
}

#[cfg(test)]
#[path = "tool_markup_tests.rs"]
mod tests;
