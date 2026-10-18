//! Qualifies the source Schedule v4 contract against the pinned v0.8.23
//! Schedule v3 artifact. This target owns a process so the existing module and
//! storage slots cannot affect another suite. Install the registry releases
//! with scripts/ci/self-hosted/install-test-modules.sh before running; no linked factory,
//! source override, missing-artifact skip or network fallback is used here.

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use openhuman_core::config::schema::ModuleSourceMode;
use openhuman_core::config::Config;
use openhuman_core::core::runtime::{
    AgentContextRegistry, ContextOverlay, CoreBuilder, CoreContext, DispatchCtx, DomainSet,
    ServiceSet,
};
use openhuman_core::cron::{self, CronJobPatch, JobPolicy, Schedule};
use openhuman_core::modules::{self, ModuleSource, ModuleState};
use openhuman_core::security::SecurityPolicy;
use openhuman_core::storage::{self, Precondition};
use openhuman_core::HostKind;
use serde_json::json;
use tinyflows_bus::schedule;

async fn assert_released_module(config: &Config) {
    assert_eq!(schedule::CONTRACT_VERSION, 4, "review the source contract");
    let record = modules::registry::find("tinyflows").unwrap();
    assert_eq!(record.version, "0.8.23");
    modules::ops::ensure_loaded(config, record.id)
        .await
        .expect("install the registry-pinned v0.8.23 TinyFlows release before this test");
    let status = modules::ops::list(config)
        .into_iter()
        .find(|status| status.id == record.id)
        .unwrap();
    assert_eq!(status.state, ModuleState::Ready);
    assert_eq!(status.source, Some(ModuleSource::Release));
    let info = modules::runtime::runtime()
        .await
        .unwrap()
        .host()
        .list()
        .into_iter()
        .find(|info| info.manifest.bus_name.as_str() == record.bus_name)
        .expect("the real module is loaded on the host bus");
    assert_eq!(info.version, record.version);
    assert_eq!(info.manifest.module.version.to_string(), record.version);
    // This release advertises its package version for each interface, not a
    // separate Schedule contract number. v0.8.23 is the pinned v3 artifact.
    let provided = info
        .manifest
        .provides
        .iter()
        .find(|provided| provided.version.interface.as_str() == schedule::INTERFACE)
        .expect("the release advertises Schedule");
    let actual: BTreeSet<_> = provided
        .methods
        .iter()
        .map(|method| method.as_str())
        .collect();
    let expected: BTreeSet<_> = schedule::METHODS.iter().copied().collect();
    assert_eq!(
        actual, expected,
        "source and release must keep the same members"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn released_schedule_v3_serves_the_host_v4_subset_and_keeps_policy() {
    let workspace = tempfile::tempdir().unwrap();
    let mut config = Config {
        workspace_dir: workspace.path().join("workspace"),
        action_dir: workspace.path().join("actions"),
        config_path: workspace.path().join("config.toml"),
        ..Config::default()
    };
    config.modules.source = ModuleSourceMode::Release;
    config.modules.overrides.clear();
    config.modules.allow_download = false;
    config.reliability.scheduler_retries = 0;
    let runtime = CoreBuilder::new(HostKind::Library)
        .config(config.clone())
        .services(ServiceSet::none())
        .domains(DomainSet::none())
        .build()
        .await
        .unwrap();
    storage::install(Arc::new(storage::MemoryStorage::new()));
    assert_released_module(&config).await;

    let security = Arc::new(SecurityPolicy::from_config(
        &config.autonomy,
        &config.workspace_dir,
        &config.action_dir,
    ));
    let context = runtime.context().derive_with(
        ContextOverlay::new(config.clone(), DomainSet::none(), Default::default())
            .session_agent("schedule-release-compatibility")
            .agent_policy(security),
    );
    AgentContextRegistry::register("schedule-release-compatibility", &context);
    let calls = Arc::new(AtomicUsize::new(0));
    let fail = Arc::new(AtomicBool::new(true));
    let (recorded, should_fail) = (Arc::clone(&calls), Arc::clone(&fail));
    let _handler = cron::system_job_handlers::register(
        "release-compatibility",
        Arc::new(move |_| {
            recorded.fetch_add(1, Ordering::SeqCst);
            let fail = should_fail.load(Ordering::SeqCst);
            Box::pin(async move {
                if fail {
                    Err("release fixture failure".to_string())
                } else {
                    Ok(())
                }
            })
        }),
    );

    CoreContext::scope(Arc::clone(&context), async {
        let dispatch = DispatchCtx::new(Arc::clone(&context));
        let job = cron::system_jobs::ensure_system_job(
            &config,
            "release-compatibility",
            Schedule::Every {
                every_ms: 3_600_000,
            },
        )
        .unwrap();
        cron::update_job(
            &config,
            &job.id,
            CronJobPatch {
                policy: Some(JobPolicy {
                    retries: Some(0),
                    single_flight: true,
                }),
                ..CronJobPatch::default()
            },
        )
        .unwrap();
        let (success, output) = cron::ops::run_job_now(&dispatch, &config, &job.id)
            .await
            .unwrap();
        assert!(!success);
        assert!(output.contains("release fixture failure"), "{output}");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        let runs = cron::list_runs(&config, &job.id, 10).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "error");

        // The v0.8.23 release owns cron_jobs and keeps all three instant
        // fields. The newer source uses cron.jobs and milliseconds alone;
        // the host document port deliberately admits both layouts. Make the
        // released fixture due without sleeping or requesting v4's clock.
        fail.store(false, Ordering::SeqCst);
        let docs = storage::current_scoped().unwrap().unwrap();
        let mut stored = docs
            .documents()
            .get("cron_jobs", &job.id)
            .await
            .unwrap()
            .unwrap();
        let due = chrono::Utc::now() - chrono::Duration::seconds(1);
        stored.doc["next_run"] = json!(due.to_rfc3339());
        stored.doc["next_run_ms"] = json!(due.timestamp_millis());
        stored.doc["next_run_ns"] = json!(due.timestamp_nanos_opt().unwrap());
        docs.documents()
            .put("cron_jobs", &job.id, stored.doc, Precondition::None)
            .await
            .unwrap();
        cron::scheduler::run_live_agent_pass(1).await;
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "the release ran the due job"
        );
        let runs = cron::list_runs(&config, &job.id, 10).unwrap();
        assert_eq!(runs.len(), 2);
        assert_eq!(runs.iter().filter(|run| run.status == "ok").count(), 1);
        assert_eq!(
            cron::get_job(&config, &job.id)
                .unwrap()
                .last_status
                .as_deref(),
            Some("ok")
        );

        let mut restricted = config.clone();
        restricted.cron.agent_jobs_only = true;
        let refused = cron::ops::run_job_now(&dispatch, &restricted, &job.id)
            .await
            .unwrap_err();
        assert!(refused.contains("only agent cron jobs"), "{refused}");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "operator refusal must not execute"
        );
        assert_eq!(cron::list_runs(&config, &job.id, 10).unwrap().len(), 2);
    })
    .await;
}
