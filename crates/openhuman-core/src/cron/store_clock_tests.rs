use super::*;
use crate::storage::{
    CollectionSpec, DocumentStore, MemoryStorage, Precondition, Scope, StorageBackend,
};
use serde_json::{json, Value};
use std::sync::Arc;

fn documents() -> Arc<dyn DocumentStore> {
    Arc::clone(
        MemoryStorage::new()
            .for_scope(&Scope::local())
            .unwrap()
            .documents(),
    )
}

async fn seed(docs: &dyn DocumentStore, collection: &str, doc: Value) {
    docs.ensure_collection(&CollectionSpec::new(collection))
        .await
        .unwrap();
    docs.put(collection, "job", doc, Precondition::Absent)
        .await
        .unwrap();
}

fn due() -> DateTime<Utc> {
    "2026-10-18T12:34:56.123456789Z".parse().unwrap()
}

#[tokio::test]
async fn a_schema_v2_clock_update_keeps_the_row_in_its_existing_collection() {
    let docs = documents();
    seed(
        docs.as_ref(),
        "cron.jobs",
        json!({"next_run_ms": 0, "name": "kept"}),
    )
    .await;

    force_next_run_in_documents_for_test(docs.as_ref(), "job", due())
        .await
        .unwrap();

    let stored = docs.get("cron.jobs", "job").await.unwrap().unwrap();
    assert_eq!(
        stored.doc,
        json!({"next_run_ms": due().timestamp_millis(), "name": "kept"})
    );
    assert!(docs.get("cron_jobs", "job").await.unwrap().is_none());
}

#[tokio::test]
async fn a_released_clock_update_keeps_all_three_instant_fields_consistent() {
    let docs = documents();
    seed(
        docs.as_ref(),
        "cron_jobs",
        json!({
            "next_run": "2000-01-01T00:00:00+00:00",
            "next_run_ms": 0,
            "next_run_ns": 0,
            "name": "kept"
        }),
    )
    .await;

    force_next_run_in_documents_for_test(docs.as_ref(), "job", due())
        .await
        .unwrap();

    let stored = docs.get("cron_jobs", "job").await.unwrap().unwrap();
    assert_eq!(
        stored.doc,
        json!({
            "next_run": due().to_rfc3339(),
            "next_run_ms": due().timestamp_millis(),
            "next_run_ns": due().timestamp_nanos_opt().unwrap(),
            "name": "kept"
        })
    );
    assert!(docs.get("cron.jobs", "job").await.unwrap().is_none());
}

#[tokio::test]
async fn a_missing_job_is_not_created_by_the_test_clock() {
    let docs = documents();
    let error = force_next_run_in_documents_for_test(docs.as_ref(), "job", due())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("no job job"));
    for collection in ["cron.jobs", "cron_jobs"] {
        assert!(docs.get(collection, "job").await.unwrap().is_none());
    }
}

#[tokio::test]
async fn an_ambiguous_job_is_refused_without_changing_either_schema() {
    let docs = documents();
    let original = json!({"next_run_ms": 0});
    for collection in ["cron.jobs", "cron_jobs"] {
        seed(docs.as_ref(), collection, original.clone()).await;
    }

    let error = force_next_run_in_documents_for_test(docs.as_ref(), "job", due())
        .await
        .unwrap_err();
    assert!(error.to_string().contains("both schedule schemas"));
    for collection in ["cron.jobs", "cron_jobs"] {
        let stored = docs.get(collection, "job").await.unwrap().unwrap();
        assert_eq!(stored.doc, original);
        assert_eq!(stored.version, tinystoragedrivers::Version::FIRST);
    }
}
