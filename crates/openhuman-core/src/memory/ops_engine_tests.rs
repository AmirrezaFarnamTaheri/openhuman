use super::*;

#[test]
fn engine_set_validates_and_rebinds() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = config_in(&tmp);

    let unknown = EngineSetParams {
        engine: "nope".into(),
        endpoint: None,
        api_key: None,
    };
    assert_eq!(
        apply_engine_set(&mut config, &unknown).unwrap_err().code(),
        "INVALID_REQUEST"
    );

    for endpoint in ["not a url", "ftp://host", "file:///etc"] {
        let params = EngineSetParams {
            engine: CORTEXDB_ENGINE.into(),
            endpoint: Some(endpoint.into()),
            api_key: None,
        };
        assert_eq!(
            apply_engine_set(&mut config, &params).unwrap_err().code(),
            "INVALID_REQUEST",
            "{endpoint}"
        );
    }

    let keyed_tinyhumans = EngineSetParams {
        engine: TINYHUMANS_ENGINE.into(),
        endpoint: None,
        api_key: Some("k".into()),
    };
    assert_eq!(
        apply_engine_set(&mut config, &keyed_tinyhumans)
            .unwrap_err()
            .code(),
        "INVALID_REQUEST"
    );

    assert!(!engine::is_on(&config));
    let cortex = EngineSetParams {
        engine: CORTEXDB_ENGINE.into(),
        endpoint: Some("https://cortex.example.test".into()),
        api_key: Some("cdb-test-key".into()),
    };
    apply_engine_set(&mut config, &cortex).unwrap();
    assert_eq!(config.memory.engine, CORTEXDB_ENGINE);
    assert_eq!(
        config.memory.endpoint_for(CORTEXDB_ENGINE).as_deref(),
        Some("https://cortex.example.test")
    );
    let bound = engine::resolve(&config).engine().expect("rebuilt on");
    assert_eq!(bound.id, CORTEXDB_ENGINE);
    assert_eq!(bound.endpoint, "https://cortex.example.test");

    let clear = EngineSetParams {
        engine: CORTEXDB_ENGINE.into(),
        endpoint: Some(String::new()),
        api_key: Some(String::new()),
    };
    apply_engine_set(&mut config, &clear).unwrap();
    assert!(config.memory.endpoint_for(CORTEXDB_ENGINE).is_none());
    assert!(!engine::is_on(&config), "removing the key turns memory off");
}

#[tokio::test]
async fn disabling_memory_turns_it_off_and_keeps_the_engine_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = config_in(&tmp);
    apply_engine_set(
        &mut config,
        &EngineSetParams {
            engine: CORTEXDB_ENGINE.into(),
            endpoint: Some("https://cortex.example.test".into()),
            api_key: Some("cdb-test-key".into()),
        },
    )
    .unwrap();
    assert!(engine::is_on(&config));

    let keyed = EngineSetParams {
        engine: DISABLED_ENGINE.into(),
        endpoint: None,
        api_key: Some("k".into()),
    };
    assert_eq!(
        apply_engine_set(&mut config, &keyed).unwrap_err().code(),
        "INVALID_REQUEST"
    );
    assert!(engine::is_on(&config), "a refused disable changes nothing");

    let disable = EngineSetParams {
        engine: DISABLED_ENGINE.into(),
        endpoint: None,
        api_key: None,
    };
    apply_engine_set(&mut config, &disable).unwrap();
    assert_eq!(config.memory.engine, DISABLED_ENGINE);
    assert!(!engine::is_on(&config));
    assert_eq!(
        config.memory.endpoint_for(CORTEXDB_ENGINE).as_deref(),
        Some("https://cortex.example.test"),
        "the cortexdb endpoint survives disabling"
    );

    let view = engine_get(&config).await;
    assert_eq!(view.engine.as_deref(), Some(DISABLED_ENGINE));
    assert_eq!(view.status, EngineStatus::Off);
    assert_eq!(view.reason.as_deref(), Some("memory is disabled"));
    assert!(engines_list(&config).active.is_none());

    // Selecting cortexdb again needs no endpoint or key re-entry.
    apply_engine_set(
        &mut config,
        &EngineSetParams {
            engine: CORTEXDB_ENGINE.into(),
            endpoint: None,
            api_key: None,
        },
    )
    .unwrap();
    assert!(engine::is_on(&config));
}

#[tokio::test]
async fn fetch_refuses_a_mode_the_engine_does_not_declare() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = config_in(&tmp);
    apply_engine_set(
        &mut config,
        &EngineSetParams {
            engine: CORTEXDB_ENGINE.into(),
            endpoint: Some("https://cortex.example.test".into()),
            api_key: Some("cdb-test-key".into()),
        },
    )
    .unwrap();
    let error = fetch(
        &config,
        FetchParams {
            refers_to: None,
            query: "x".into(),
            mode: Some(FetchMode::Keyword),
            filter: None,
            limit: None,
            cursor: None,
        },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code(), "UNSUPPORTED");
}

#[tokio::test]
async fn erase_all_needs_its_confirmation() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_in(&tmp);
    let engine = bind_reference(&config);
    learn(&config, learn_params("a fact"), None).await.unwrap();
    let error = erase_all(
        &crate::core::runtime::DispatchCtx::ambient(),
        &config,
        EraseAllParams { confirm: false },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code(), "INVALID_REQUEST");
    assert_eq!(stored(&engine, MetaFilter::default()).await.len(), 1);
}

#[tokio::test]
async fn erase_all_erases_everything_the_engine_holds() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_in(&tmp);
    let engine = bind_reference(&config);
    learn(&config, learn_params("a fact"), None).await.unwrap();
    learn(&config, learn_params("another fact"), None)
        .await
        .unwrap();
    let view = erase_all(
        &crate::core::runtime::DispatchCtx::ambient(),
        &config,
        EraseAllParams { confirm: true },
    )
    .await
    .unwrap();
    assert!(view.erased_scopes >= 1, "{view:?}");
    assert!(stored(&engine, MetaFilter::default()).await.is_empty());
}

#[tokio::test]
async fn erase_all_reports_memory_off_without_an_engine() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_in(&tmp);
    let error = erase_all(
        &crate::core::runtime::DispatchCtx::ambient(),
        &config,
        EraseAllParams { confirm: true },
    )
    .await
    .unwrap_err();
    assert_eq!(error.code(), "MEMORY_OFF");
}

#[tokio::test]
async fn erasing_memory_cancels_queued_learning_so_restart_cannot_restore_it() {
    let tmp = tempfile::tempdir().unwrap();
    let config = config_in(&tmp);
    let engine = bind_reference(&config);
    let facts = crate::memory::tools::CallFacts::of(
        &config,
        &tinymemory_tools::layout::identity::Identity::agent("orchestrator"),
    );
    let queued = crate::memory::tool_writes::tests::enqueue(
        &config,
        &serde_json::json!({"action":"learn","text":"Prefers tea"}),
        &facts,
    )
    .unwrap();
    assert!(queued["status"].as_str().unwrap().starts_with("queued"));
    erase_all(
        &crate::core::runtime::DispatchCtx::ambient(),
        &config,
        EraseAllParams { confirm: true },
    )
    .await
    .unwrap();
    let restarted = config.clone();
    crate::memory::tool_writes::drain(&restarted).await;
    assert!(stored(&engine, MetaFilter::default()).await.is_empty());
}

/// With a reach, forget by id goes to the engine's `forget_within` (through
/// the scrubbing wrapper every bound engine sits in), so the engine never
/// sweeps the tree for the ids; without one it stays a plain forget by id.
#[tokio::test]
async fn forget_with_a_reach_forgets_within_it() {
    use tinymemory_api::MemoryEngine as _;

    let tmp = tempfile::tempdir().unwrap();
    let config = config_in(&tmp);
    let engine = std::sync::Arc::new(crate::memory::test_fixtures::RecordingEngine::new());
    crate::memory::test_fixtures::RecordingEngine::bind(&engine, &config);

    let mine = learn(&config, learn_params("tea in the morning"), None)
        .await
        .unwrap();
    let other = learn(&config, learn_params("coffee at night"), None)
        .await
        .unwrap();
    let namespace = engine
        .list(ListRequest {
            filter: MetaFilter::default(),
            limit: 10,
            cursor: None,
        })
        .await
        .unwrap()
        .items
        .into_iter()
        .find(|hit| hit.id.0 == mine.id)
        .expect("learned item listed")
        .meta
        .namespace;

    let forgotten = forget(
        &config,
        ForgetParams {
            ids: vec![mine.id.clone()],
            reach: Some(tinymemory_api::Reach::exact(namespace)),
        },
    )
    .await
    .unwrap();
    assert_eq!(forgotten.forgotten, 1);
    assert_eq!(engine.calls(), vec!["forget_within"]);

    let forgotten = forget(
        &config,
        ForgetParams {
            ids: vec![other.id.clone()],
            reach: None,
        },
    )
    .await
    .unwrap();
    assert_eq!(forgotten.forgotten, 1);
    assert_eq!(engine.calls(), vec!["forget_within", "forget"]);
}

#[path = "ops_compat_tests.rs"]
mod compat_tests;

#[test]
fn disabled_engine_rejects_an_api_key_without_mutating_memory_settings() {
    let tmp = tempfile::tempdir().unwrap();
    let mut config = config_in(&tmp);
    let before = config.memory.clone();

    let error = apply_engine_set(
        &mut config,
        &EngineSetParams {
            engine: DISABLED_ENGINE.into(),
            endpoint: None,
            api_key: Some("not-a-real-key".into()),
        },
    )
    .unwrap_err();

    assert_eq!(error.code(), "INVALID_REQUEST");
    assert_eq!(config.memory, before);
}
