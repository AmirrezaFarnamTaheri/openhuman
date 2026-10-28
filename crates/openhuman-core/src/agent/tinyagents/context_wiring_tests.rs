use super::*;
use tinyagents_harness::turn_outcome::ToolCallOutcome;

#[test]
fn outcome_source_returns_captured_content_by_call_id() {
    let sink: ToolOutcomeSink = Default::default();
    let source = OutcomeSinkSource(sink.clone());
    assert!(source.content_for("missing").unwrap().is_none());
    sink.lock().unwrap().push(ToolCallOutcome {
        call_id: "c1".into(),
        name: "t".into(),
        arguments: serde_json::Value::Null,
        success: true,
        content: "body".into(),
        duration_ms: 0,
    });
    assert_eq!(source.content_for("c1").unwrap().as_deref(), Some("body"));
}

#[test]
fn budget_notice_thresholds_stay_at_half_and_eighty_percent() {
    assert_eq!(BUDGET_NOTICE_THRESHOLDS, [0.5, 0.8]);
}

/// Records each call it gets, then fails: the tests read the order only.
struct RecordingSummarizer(Arc<std::sync::Mutex<Vec<&'static str>>>);

#[async_trait]
impl Summarizer for RecordingSummarizer {
    async fn summarize(&self, _: &[Message]) -> tinyagents_harness::error::Result<SummaryRecord> {
        self.0.lock().unwrap().push("summarize");
        Err(tinyagents_harness::error::TinyAgentsError::Model(
            "stub".into(),
        ))
    }

    async fn merge(&self, _: &[SummaryRecord]) -> tinyagents_harness::error::Result<SummaryRecord> {
        self.0.lock().unwrap().push("merge");
        Err(tinyagents_harness::error::TinyAgentsError::Model(
            "stub".into(),
        ))
    }
}

fn recording() -> (
    Arc<std::sync::Mutex<Vec<&'static str>>>,
    Box<dyn Summarizer>,
) {
    let calls = Arc::new(std::sync::Mutex::new(Vec::new()));
    let notice_calls = calls.clone();
    let notice: CompactionNotice = Arc::new(move || notice_calls.lock().unwrap().push("notice"));
    let summarizer = announced(Box::new(RecordingSummarizer(calls.clone())), Some(notice));
    (calls, summarizer)
}

#[tokio::test]
async fn a_summarization_is_announced_before_it_runs() {
    let (calls, summarizer) = recording();
    let _ = summarizer.summarize(&[]).await;
    let _ = summarizer
        .summarize_request(&SummaryRequest::new(Vec::new()))
        .await;
    assert_eq!(
        *calls.lock().unwrap(),
        ["notice", "summarize", "notice", "summarize"]
    );
}

#[tokio::test]
async fn merging_summaries_is_not_announced_again() {
    let (calls, summarizer) = recording();
    let _ = summarizer.merge(&[]).await;
    assert_eq!(*calls.lock().unwrap(), ["merge"]);
}

/// A compaction names the summarizer's own model in its provenance, not the
/// turn's: with `[context] summarizer_model` set the two differ.
#[tokio::test]
async fn a_compaction_records_the_summarizer_model_it_called() {
    use tinyagents_harness::context::{RunConfig, RunContext};
    use tinyagents_harness::testkit::ScriptedModel;

    let state_model = Arc::new(ScriptedModel::replies(vec![
        "{\"goal\": \"keep going\"}";
        4
    ]));
    let mw = Arc::new(compression_middleware(
        SummarizationPolicy::default().with_trigger_override(200),
        crate::config::CompactionStrategy::TaskState,
        None,
        state_model,
        "scaledown/summarize",
        "turn-model",
        None,
        None,
    ));
    let mut harness: AgentHarness<()> = AgentHarness::new();
    harness.with_model_resolver(Arc::new(
        tinyagents_harness::host::FixedModelResolver::named(
            "turn-model",
            Arc::new(ScriptedModel::replies(vec!["done"])) as _,
        ),
    ));
    harness.push_middleware(mw.clone());
    harness.push_model_middleware(mw.clone());

    let mut history = vec![Message::system("system")];
    for round in 0..6 {
        history.push(Message::user(format!("ask {round}: {}", "detail ".repeat(60))));
        history.push(Message::assistant(format!(
            "answer {round}: {}",
            "result ".repeat(60)
        )));
    }
    history.push(Message::user("continue"));
    harness
        .invoke_in_context(&(), RunContext::new(RunConfig::new("compaction"), ()), history)
        .await
        .expect("the turn runs");

    let records = mw.records();
    assert!(!records.is_empty(), "the turn compacted");
    for record in records {
        assert!(
            record.provenance.reason.contains("via scaledown/summarize"),
            "{}",
            record.provenance.reason
        );
    }
}
