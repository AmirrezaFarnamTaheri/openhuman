//! [`TinyJevEvaluator`]: the `tinytools_jev::JevEvaluator` over
//! `tinyjevclient`, reaching Jev through the TinyHumans System One proxy.
//!
//! `tinytools-jev` decides *what* to ask — the options, the intent, the
//! family-stage wording — and this evaluator owns the wire: one `Choice`
//! over the options plus a `needs_tool` `Noul`, the credential, the
//! per-attempt timeout and retries the client applies, and a deadline of
//! its own so a slow answer becomes a fallback rather than a stalled turn.

use std::{collections::{BTreeMap, HashSet}, time::Duration};

use serde_json::{json, Value};
use tinyjevclient::{
    Answer, Choice, Client, Error as JevError, EvaluationFailure, EvaluationRequest, Noul,
    NoulCriteria, Question,
};
use tinytools::RankError;
use tinytools_jev::{JevDecision, JevEvaluator, JevRequest};

/// Question id of the tool `Choice`.
const TOOL_QUESTION: &str = "tool";
/// Question id of the needs-a-tool `Noul`.
const NEEDS_TOOL_QUESTION: &str = "needs_tool";
/// Default deadline for one evaluation, on top of the client's own
/// per-attempt timeout and retries.
const DEFAULT_DEADLINE: Duration = Duration::from_secs(3);

/// Keep externally routed System-One context bounded. Preserve the latest
/// user turns in chronological order, with explicit partial metadata; do not
/// forward entire conversations, tool results, or internal agent transcripts.
const MAX_DECISION_RECENT_TURNS: usize = 6;
const MAX_DECISION_TURN_CHARS: usize = 500;
const MAX_DECISION_INTENT_CHARS: usize = 4_000;
const MAX_DECISION_OPTIONS: usize = 255;
const MAX_DECISION_TOTAL_DESCRIPTION_CHARS: usize = 96_000;
const MAX_DECISION_OPTION_DESCRIPTION_CHARS: usize = 2_000;

/// Evaluates `tinytools_jev` requests against Jev through `tinyjevclient`.
#[derive(Debug, Clone)]
pub struct TinyJevEvaluator {
    client: Client,
    deadline: Duration,
}

impl TinyJevEvaluator {
    /// An evaluator over `client` with the default 3 s deadline.
    pub fn new(client: Client) -> Self {
        Self {
            client,
            deadline: DEFAULT_DEADLINE,
        }
    }

    /// Sets the per-evaluation deadline.
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }

    /// Enforce the option identity and context budget before remote egress.
    /// A duplicate key would otherwise overwrite a Choice criterion in the
    /// BTreeMap and silently change which tool System-One is choosing from.
    fn validate(request: &JevRequest) -> Result<(), RankError> {
        if request.options.is_empty() || request.options.len() > MAX_DECISION_OPTIONS {
            return Err(RankError::invalid_input(
                "System-One requires 1 to 255 distinct tool choices".to_owned()
            ));
        }
        let mut keys = HashSet::with_capacity(request.options.len());
        let mut description_total = 0usize;
        for option in &request.options {
            if option.key.trim().is_empty()
                || option.key.len() > 256
                || option.key.chars().any(char::is_control)
                || !keys.insert(option.key.as_str())
            {
                return Err(RankError::invalid_input(
                    "System-One contains an invalid or duplicate tool option key".to_owned()
                ));
            }
            let characters = option.description.chars()
                .take(MAX_DECISION_OPTION_DESCRIPTION_CHARS + 1).count();
            if characters > MAX_DECISION_OPTION_DESCRIPTION_CHARS {
                return Err(RankError::invalid_input(
                    "System-One tool option description exceeds the decision budget".to_owned()
                ));
            }
            description_total += characters;
            if description_total > MAX_DECISION_TOTAL_DESCRIPTION_CHARS {
                return Err(RankError::invalid_input(
                    "System-One cumulative tool-choice description exceeds the budget".to_owned()
                ));
            }
        }
        Ok(())
    }

    fn build(request: &JevRequest) -> EvaluationRequest {
        // Preserve both the original goal and the latest specified conditions
        // when a tool-search intent is abnormally long. The marker is explicit
        // to the decision model; this is a size guard, not semantic summarization.
        let intent_truncated = request.intent.chars().count() > MAX_DECISION_INTENT_CHARS;
        let intent = if intent_truncated {
            let head: String = request.intent.chars()
                .take(MAX_DECISION_INTENT_CHARS / 2).collect();
            let tail: String = request.intent.chars().rev()
                .take(MAX_DECISION_INTENT_CHARS / 2)
                .collect::<Vec<_>>().into_iter().rev().collect();
            format!("{head}\n[... middle of request omitted due to context budget ...]\n{tail}")
        } else {
            request.intent.clone()
        };
        let mut state = json!({ "request": intent });
        // Jev receives only task intent and bounded user-turn context, never
        // the raw system prompt, history, tool results or credentials. Choosing
        // the latest turns matches how tool choice should use recent intent.
        let candidates: Vec<_> = request.recent_turns.iter()
            .filter(|turn| !turn.trim().is_empty())
            .collect();
        let offset = candidates.len().saturating_sub(MAX_DECISION_RECENT_TURNS);
        let selected = &candidates[offset..];
        let mut truncated = offset != 0 || intent_truncated;
        let recent: Vec<Value> = selected.iter().map(|turn| {
            if turn.chars().count() > MAX_DECISION_TURN_CHARS {
                truncated = true;
            }
            Value::String(turn.chars().take(MAX_DECISION_TURN_CHARS).collect())
        }).collect();
        if let Some(object) = state.as_object_mut() {
            if !recent.is_empty() {
                object.insert("recent_user_turns".to_owned(), Value::Array(recent));
            }
            if truncated {
                object.insert("context_truncated".to_owned(), Value::Bool(true));
                if offset > 0 {
                    object.insert("recent_turns_dropped".to_owned(), json!(offset));
                }
                if intent_truncated {
                    object.insert("request_truncated".to_owned(), Value::Bool(true));
                }
            }
        }
        let criteria: BTreeMap<String, Option<Value>> = request
            .options
            .iter()
            .map(|option| (option.key.clone(), Some(json!(option.description))))
            .collect();
        let instructions = request.instructions.clone().unwrap_or_else(|| {
            "Which tool accomplishes the user's `request`? Judge by what each tool does, \
             not by shared words. Pick `none` when no listed tool does it."
                .to_owned()
        });
        EvaluationRequest {
            state,
            model: request.model.clone(),
            questions: BTreeMap::from([
                (
                    TOOL_QUESTION.to_owned(),
                    Question::Choice(Choice {
                        instructions: json!(instructions),
                        criteria,
                    }),
                ),
                (
                    NEEDS_TOOL_QUESTION.to_owned(),
                    Question::Noul(Noul {
                        instructions: json!(
                            "Does fulfilling the user's `request` require calling a tool \
                             — an action or a lookup outside the assistant's own knowledge?"
                        ),
                        criteria: Some(NoulCriteria {
                            r#true: json!(
                                "The request asks for an action or for information that \
                                 must be fetched."
                            ),
                            r#false: json!(
                                "The request can be answered by replying, with no tool."
                            ),
                        }),
                    }),
                ),
            ]),
        }
    }
}

fn map_failure(failure: EvaluationFailure) -> RankError {
    match failure.error {
        JevError::InvalidRequest { reason } | JevError::InvalidConfig { reason } => {
            RankError::invalid_input(reason)
        }
        JevError::Timeout => RankError::Timeout,
        // `Display` on every variant is credential-free by the client's
        // contract; the transport source is dropped, not printed.
        other => RankError::backend(format!("{other} after {} attempt(s)", failure.attempts)),
    }
}

#[async_trait::async_trait]
impl JevEvaluator for TinyJevEvaluator {
    async fn evaluate(&self, request: &JevRequest) -> Result<JevDecision, RankError> {
        Self::validate(request)?;
        let wire = Self::build(request);
        let result = tokio::time::timeout(self.deadline, self.client.evaluate(&wire))
            .await
            .map_err(|_elapsed| RankError::Timeout)?
            .map_err(map_failure)?;
        let Some(Answer::Choice(choice)) = result.response.answers.get(TOOL_QUESTION) else {
            return Err(RankError::backend(
                "response has no choice answer for `tool`",
            ));
        };
        let needs_tool = match result.response.answers.get(NEEDS_TOOL_QUESTION) {
            Some(Answer::Noul(noul)) => Some(noul.noul),
            _ => None,
        };
        log::debug!(
            "[tool-search] jev evaluated {} option(s) (confidence={:.2} needs_tool={:?} latency_ms={} attempts={} input_tokens={:?})",
            request.options.len(),
            choice.confidence,
            needs_tool,
            result.latency.as_millis(),
            result.attempts,
            result.response.usage.input_tokens,
        );
        Ok(JevDecision {
            probabilities: choice.probabilities.clone(),
            choice_confidence: choice.confidence,
            needs_tool,
            input_tokens: result.response.usage.input_tokens,
            attempts: result.attempts,
        })
    }
}

#[cfg(test)]
#[path = "evaluator_tests.rs"]
mod tests;
