use super::*;
use serde_json::json;
use tinyflows::engine::GraphObservation;
use tinyflows::graph::stream::GraphEvent;

fn envelope(event: Value) -> Value {
    json!({
        "event_id": "event", "run_id": "run", "root_run_id": "root",
        "graph_id": "graph", "step": 1, "offset": 2, "ts_ms": 3, "event": event
    })
}

#[test]
fn every_engine_event_keeps_its_wire_payload_and_trace_kind() {
    let data = json!({
        "run_id": "run", "steps": 1, "step": 1, "active": ["node"],
        "node": "node", "error": "failed", "attempt": 2, "target": "next",
        "checkpoint_id": "checkpoint", "namespace": ["child"], "fork": 0,
        "depth": 1, "name": "custom", "data": {"nested": [1, 2]},
        "interrupt": {"id": "pause", "node": "node", "payload": {"question": "continue?"}}
    });
    for name in [
        "RunStarted",
        "RunCompleted",
        "RunFailed",
        "StepStarted",
        "StepCompleted",
        "TaskScheduled",
        "NodeStarted",
        "NodeCompleted",
        "NodeFailed",
        "NodeRetryScheduled",
        "StateUpdated",
        "RouteSelected",
        "CheckpointSaved",
        "CheckpointRestored",
        "InterruptEmitted",
        "SubgraphStarted",
        "SubgraphCompleted",
        "ContextForked",
        "RecursionDepthChanged",
        "Custom",
    ] {
        let event: GraphEvent = serde_json::from_value(Value::Object(
            [(name.to_owned(), data.clone())].into_iter().collect(),
        ))
        .unwrap();
        let engine: GraphObservation = serde_json::from_value(envelope(json!(event))).unwrap();
        let wire = serde_json::to_value(engine).unwrap();
        let projected: TraceObservation = serde_json::from_value(wire.clone()).unwrap();
        assert_eq!(projected.event_kind(), event.kind(), "{name}");
        assert_eq!(serde_json::to_value(projected).unwrap(), wire, "{name}");
    }
}

#[test]
fn future_events_and_fields_remain_available_to_trace_metadata() {
    let event = json!({"FutureEvent": {"nested": [1, {"field": true}]}});
    let projected: TraceObservation = serde_json::from_value(envelope(event.clone())).unwrap();
    assert_eq!(projected.event_name(), "FutureEvent");
    assert_eq!(projected.event, event);
}

#[test]
fn malformed_event_envelopes_are_rejected() {
    for event in [
        json!([]),
        json!({}),
        json!({"A": {}, "B": {}}),
        json!({"A": null}),
    ] {
        assert!(serde_json::from_value::<TraceObservation>(envelope(event)).is_err());
    }
}
