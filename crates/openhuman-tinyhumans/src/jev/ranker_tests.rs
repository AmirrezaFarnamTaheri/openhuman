use tinytools::{RankCandidate, RankContext, RankError, ToolRanker};

use super::*;

#[test]
fn kind_is_jev_and_debug_hides_the_client() {
    let ranker = TinyHumansJevRanker::new();
    assert_eq!(ranker.kind(), "jev");
    let debug = format!("{ranker:?}");
    assert!(debug.contains("TinyHumansJevRanker"));
    assert!(!debug.contains("api_key"));
}

#[test]
fn fingerprint_changes_with_secret_or_base_url() {
    let a = fingerprint("secret", "https://api.tinyhumans.ai");
    assert_eq!(a, fingerprint("secret", "https://api.tinyhumans.ai"));
    assert_ne!(a, fingerprint("other", "https://api.tinyhumans.ai"));
    assert_ne!(a, fingerprint("secret", "https://staging.tinyhumans.ai"));
    assert!(!fingerprint_changed(None, a));
}

/// Without a credential the ranker must answer with a `Backend` error the
/// harness turns into a BM25 fallback — never a panic, never a network call.
#[tokio::test]
async fn no_credential_is_a_backend_error_naming_the_gap() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    // No env keys: a developer's shell may export OPENROUTER_API_KEY, and `auto`
    // would then (correctly) route through it.
    let ranker = TinyHumansJevRanker::new()
        .with_env_loader(Arc::new(|_| None))
        .with_config_loader(Arc::new(move || {
            let config = config.clone();
            Box::pin(async move { Ok(config) })
        }));
    let err = ranker
        .rank(
            "send a message",
            &RankContext::empty(),
            &[RankCandidate::new("a", "a send a message")],
            3,
        )
        .await
        .expect_err("no credential must fail");
    match err {
        RankError::Backend { reason } => {
            assert!(reason.contains("no TinyHumans credential"), "{reason}");
        }
        other => panic!("expected a backend error, got {other}"),
    }
}

/// A config whose embedding provider is `none` disables the Jev search
/// outright — the harness's BM25 answers — rather than quietly retrieving
/// lexically.
#[test]
fn no_embedding_provider_disables_the_search() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    config.memory.embedding_provider = "none".into();
    let err = retriever_for(&config).err().map(|e| e.to_string());
    assert!(
        err.as_deref()
            .is_some_and(|e| e.contains("no usable embedding provider")),
        "{err:?}"
    );
}


#[test]
fn transient_jev_failures_trip_a_bounded_provider_circuit() {
    let start = std::time::Instant::now();
    let mut circuit = JevCircuit::default();
    assert!(circuit.can_attempt(17, start));
    circuit.record_failure(17, start);
    circuit.record_failure(17, start);
    assert!(circuit.can_attempt(17, start), "first two failures may retry");
    circuit.record_failure(17, start);
    assert!(!circuit.can_attempt(17, start), "third failure should short circuit");
    assert!(!circuit.can_attempt(17, start + Duration::from_secs(19)));
    assert!(circuit.can_attempt(17, start + Duration::from_secs(21)));
}

#[test]
fn jev_circuit_resets_after_success_and_credential_route_rotation() {
    let start = std::time::Instant::now();
    let mut circuit = JevCircuit::default();
    for _ in 0..3 { circuit.record_failure(10, start); }
    assert!(!circuit.can_attempt(10, start));
    assert!(circuit.can_attempt(11, start), "new route/credential may recover");
    circuit.record_failure(11, start);
    circuit.record_success(11);
    assert!(circuit.can_attempt(11, start));
    circuit.record_failure(11, start);
    circuit.record_failure(11, start);
    assert!(circuit.can_attempt(11, start), "success clears prior streak");
}
