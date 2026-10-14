//! Strict tagged-JSON text tool protocol from 2API peers (web2api).
//!
//! Supported envelopes:
//! <think>...</think><tool_calls>[{"name":"read_file","arguments":{...}}]</tool_calls>
//! <think>...</think><tool_call>{"name":"read_file","arguments":{...}}</tool_call>
//! <think>...</think><final_answer>...</final_answer>
//!
//! This is a *decoder only*. It never creates tool declarations or bypasses
//! permissions. Malformed tagged output fails closed instead of being repaired.

use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;
use tinyinference_llm::message::ContentBlock;
use tinyinference_llm::model::ModelResponse;
use tinyinference_llm::tool::{ToolCall, ToolSchema};

use super::{validate_native_tool_calls, MAX_OUTPUT_BYTES, MAX_TOOL_CALLS, MAX_VALUE_BYTES};

static NEXT_TAGGED_CALL: AtomicU64 = AtomicU64::new(1);

#[derive(Debug)]
pub(super) struct TaggedJsonOutput {
    pub(super) thinking: Option<String>,
    pub(super) calls: Vec<ToolCall>,
    pub(super) answer: Option<String>,
}

fn invalid(reason: &'static str) -> tinyinference_llm::Error {
    tinyinference_llm::Error::Validation(format!("tagged JSON tool response: {reason}"))
}

fn extract_block<'a>(
    rest: &'a str,
    open: &'static str,
    close: &'static str,
) -> tinyinference_llm::Result<(&'a str, &'a str)> {
    let inner = rest
        .strip_prefix(open)
        .ok_or_else(|| invalid("expected opening tag"))?;
    let (body, tail) = inner
        .split_once(close)
        .ok_or_else(|| invalid("unclosed tag"))?;
    Ok((body, tail.trim_start()))
}

fn decode_calls(
    raw: &str,
    is_parallel: bool,
    declared: &[ToolSchema],
) -> tinyinference_llm::Result<Vec<ToolCall>> {
    let payload: Value =
        serde_json::from_str(raw).map_err(|_| invalid("malformed JSON tool-call payload"))?;
    let rows = if is_parallel {
        payload
            .as_array()
            .ok_or_else(|| invalid("tool_calls payload must be a JSON array"))?
            .clone()
    } else {
        if !payload.is_object() {
            return Err(invalid("tool_call payload must be a JSON object"));
        }
        vec![payload]
    };
    if rows.is_empty() || rows.len() > MAX_TOOL_CALLS {
        return Err(invalid("tool call count outside 1..16"));
    }
    let mut calls = Vec::with_capacity(rows.len());
    for row in rows {
        let object = row
            .as_object()
            .ok_or_else(|| invalid("tool call must be a JSON object"))?;
        let name = object
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| invalid("tool name must be a string"))?;
        if name.is_empty() || name.len() > 128 || name.trim() != name {
            return Err(invalid("invalid tool name"));
        }
        let arguments = object
            .get("arguments")
            .and_then(Value::as_object)
            .ok_or_else(|| invalid("arguments must be a JSON object"))?;
        if arguments.len() > super::MAX_ARGUMENTS_PER_CALL {
            return Err(invalid("too many tool arguments"));
        }
        if serde_json::to_vec(arguments)
            .map_err(|_| invalid("cannot serialize tool arguments"))?
            .len() > MAX_VALUE_BYTES
        {
            return Err(invalid("tool arguments exceed size limit"));
        }
        let tool = declared
            .iter()
            .find(|tool| tool.name == name)
            .ok_or_else(|| invalid("tool not declared by request"))?;
        let seq = NEXT_TAGGED_CALL.fetch_add(1, Ordering::Relaxed);
        let call = ToolCall::new(
            format!("call_tagged_{:x}_{seq:x}", std::process::id()),
            name,
            Value::Object(arguments.clone()),
        );
        tool.validate_call(&call)?;
        calls.push(call);
    }
    Ok(calls)
}

/// Strictly parses one complete tagged protocol response. Unrecognized prose
/// is not interpreted as an executable command even if it quotes a tag.
pub(super) fn parse_tagged_json_output(
    output: &str,
    declared: &[ToolSchema],
) -> tinyinference_llm::Result<Option<TaggedJsonOutput>> {
    let trimmed = output.trim();
    if !trimmed.starts_with("<think>")
        && !trimmed.starts_with("<tool_call>")
        && !trimmed.starts_with("<tool_calls>")
        && !trimmed.starts_with("<final_answer>")
    {
        return Ok(None);
    }
    if trimmed.len() > MAX_OUTPUT_BYTES {
        return Err(invalid("response exceeds bounded buffer"));
    }
    let mut rest = trimmed;
    let mut thinking_blocks = Vec::new();
    while rest.starts_with("<think>") {
        let (body, tail) = extract_block(rest, "<think>", "</think>")?;
        if !body.trim().is_empty() {
            thinking_blocks.push(body.trim());
        }
        rest = tail;
    }
    let thinking = (!thinking_blocks.is_empty()).then(|| thinking_blocks.join("\n\n"));
    let (calls, answer, tail) = if rest.starts_with("<tool_calls>") {
        let (body, tail) = extract_block(rest, "<tool_calls>", "</tool_calls>")?;
        (decode_calls(body.trim(), true, declared)?, None, tail)
    } else if rest.starts_with("<tool_call>") {
        let (body, tail) = extract_block(rest, "<tool_call>", "</tool_call>")?;
        (decode_calls(body.trim(), false, declared)?, None, tail)
    } else if rest.starts_with("<final_answer>") {
        let (body, tail) = extract_block(rest, "<final_answer>", "</final_answer>")?;
        (Vec::new(), Some(body.trim().to_string()), tail)
    } else {
        return Err(invalid("missing terminal tool or final-answer block"));
    };
    if !tail.trim().is_empty() {
        return Err(invalid("unexpected trailing text or additional terminal block"));
    }
    Ok(Some(TaggedJsonOutput {
        thinking,
        calls,
        answer,
    }))
}

/// Rewrites only a complete tagged response into native text/thinking/tool
/// blocks. Native upstream tool calls take precedence and are not overwritten.
pub(super) fn rewrite_tagged_json_response(
    mut response: ModelResponse,
    declared: &[ToolSchema],
) -> tinyinference_llm::Result<ModelResponse> {
    if !response.message.tool_calls.is_empty() {
        validate_native_tool_calls(&response.message.tool_calls, declared)?;
        return Ok(response);
    }
    let text_indices: Vec<usize> = response
        .message
        .content
        .iter()
        .enumerate()
        .filter_map(|(index, block)| matches!(block, ContentBlock::Text(_)).then_some(index))
        .collect();
    if text_indices.len() != 1 {
        return Ok(response);
    }
    let index = text_indices[0];
    let ContentBlock::Text(ref text) = response.message.content[index] else {
        return Ok(response);
    };
    let Some(decoded) = parse_tagged_json_output(text, declared)? else {
        return Ok(response);
    };
    response.message.content.remove(index);
    let mut insert = index;
    if let Some(reasoning) = decoded.thinking {
        response.message.content.insert(
            insert,
            ContentBlock::Thinking {
                text: reasoning,
                signature: None,
            },
        );
        insert += 1;
    }
    if let Some(answer) = decoded.answer {
        response.message.content.insert(insert, ContentBlock::Text(answer));
        response.finish_reason = Some("stop".into());
    } else {
        response.message.tool_calls = decoded.calls;
        response.finish_reason = Some("tool_calls".into());
    }
    Ok(response)
}
