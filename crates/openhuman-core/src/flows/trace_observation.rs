//! The PollRun fields consumed by the host's trace exporter.
//! Keep event payloads intact without linking the module's graph engine.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TraceObservation {
    pub event_id: String,
    pub run_id: String,
    pub root_run_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,
    pub graph_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub namespace: Vec<String>,
    pub step: usize,
    pub offset: u64,
    pub ts_ms: u64,
    #[serde(deserialize_with = "deserialize_event")]
    pub event: Value,
}

fn deserialize_event<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Value, D::Error> {
    let event = Value::deserialize(deserializer)?;
    if !event.as_object().is_some_and(|object| {
        object.len() == 1
            && object
                .iter()
                .all(|(name, data)| !name.is_empty() && data.is_object())
    }) {
        return Err(serde::de::Error::custom("expected one graph event object"));
    }
    Ok(event)
}

impl TraceObservation {
    pub(crate) fn event_parts(&self) -> (&str, &Value) {
        self.event
            .as_object()
            .and_then(|object| object.iter().next())
            .map(|(name, data)| (name.as_str(), data))
            .unwrap_or(("unknown", &self.event))
    }

    pub(crate) fn event_name(&self) -> &str {
        self.event_parts().0
    }

    pub(crate) fn event_kind(&self) -> &str {
        match self.event_name() {
            "RunStarted" => "run.started",
            "RunCompleted" => "run.completed",
            "RunFailed" => "run.failed",
            "StepStarted" => "step.started",
            "StepCompleted" => "step.completed",
            "TaskScheduled" => "task.scheduled",
            "NodeStarted" => "node.started",
            "NodeCompleted" => "node.completed",
            "NodeFailed" => "node.failed",
            "NodeRetryScheduled" => "node.retry_scheduled",
            "StateUpdated" => "state.updated",
            "RouteSelected" => "route.selected",
            "CheckpointSaved" => "checkpoint.saved",
            "CheckpointRestored" => "checkpoint.restored",
            "InterruptEmitted" => "interrupt.emitted",
            "SubgraphStarted" => "subgraph.started",
            "SubgraphCompleted" => "subgraph.completed",
            "ContextForked" => "context.forked",
            "RecursionDepthChanged" => "recursion.depth_changed",
            "Custom" => "custom",
            other => other,
        }
    }
}

#[cfg(test)]
#[path = "trace_observation_tests.rs"]
mod tests;
