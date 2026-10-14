use super::*;
#[test]
fn split_deduplicates_and_discards_blanks() {
    assert_eq!(parse_api_keys(" key-a, key-b;key-a\nkey-c\r\n"), vec!["key-a", "key-b", "key-c"]);
}
#[test]
fn routes_around_cooldown_and_revoked_keys() {
    let scope = "key-pool-test-routing";
    let keys = vec!["a".to_owned(), "b".to_owned()];
    let (i, first) = next_key(scope, &keys).unwrap();
    assert_eq!(first, "a");
    record_result(scope, i, Some(KeyFailure::RateLimited));
    assert_eq!(next_key(scope, &keys).unwrap().1, "b");
    record_result(scope, 1, Some(KeyFailure::Authentication));
    assert!(next_key(scope, &keys).is_none());
    assert_eq!(health(scope).len(), 2);
}
#[test]
fn exhaustion_never_reuses_an_attempted_key() {
    let scope = "key-pool-test-exhausted";
    let keys = vec!["a".to_owned(), "b".to_owned()];
    let (first, _) = next_key_excluding(scope, &keys, &[]).unwrap();
    let (second, _) = next_key_excluding(scope, &keys, &[first]).unwrap();
    assert_ne!(first, second);
    assert!(next_key_excluding(scope, &keys, &[first, second]).is_none());
}
#[test]
fn outcome_is_attributed_by_secret_not_stale_array_position() {
    let scope = "key-pool-test-rotated";
    let initial = vec!["a".to_owned(), "b".to_owned()];
    let (_, key) = next_key(scope, &initial).unwrap();
    let updated = vec!["b".to_owned(), "a".to_owned()];
    next_key(scope, &updated).unwrap();
    record_result_for(scope, &key, Some(KeyFailure::Authentication), None);
    assert_eq!(next_key(scope, &updated).unwrap().1, "b");
}
#[test]
fn updates_pool_without_losing_existing_health() {
    let scope = "key-pool-test-update";
    let initial = vec!["one".to_owned(), "two".to_owned()];
    next_key(scope, &initial);
    record_result(scope, 0, Some(KeyFailure::Authentication));
    let updated = vec!["one".to_owned(), "three".to_owned()];
    assert_eq!(next_key(scope, &updated).unwrap().1, "three");
    assert!(health(scope)[0].0);
}
#[test]
fn one_run_reuses_healthy_key_but_fails_over_after_rate_limit() {
    let scope = "key-pool-test-session-affinity";
    let keys = vec!["key-a".to_owned(), "key-b".to_owned()];
    let (first, _) = choose_key_for_run(
        scope, &keys, &[], RotationStrategy::RoundRobin, Some("run-1")
    ).unwrap();
    let (sticky, _) = choose_key_for_run(
        scope, &keys, &[], RotationStrategy::RoundRobin, Some("run-1")
    ).unwrap();
    assert_eq!(first, sticky);
    record_result(scope, first, Some(KeyFailure::RateLimited));
    let (fallback, _) = choose_key_for_run(
        scope, &keys, &[], RotationStrategy::RoundRobin, Some("run-1")
    ).unwrap();
    assert_ne!(fallback, first);
}
#[test]
fn weighted_scoring_prefers_unleased_successful_credentials() {
    let fresh = KeyHealth::new(secret_id("fresh"));
    let mut loaded = fresh.clone();
    loaded.active = 2;
    assert!(weighted_score(&fresh) > weighted_score(&loaded));
    loaded.active = 0;
    loaded.consecutive_failures = 2;
    assert!(weighted_score(&fresh) > weighted_score(&loaded));
}
#[test]
fn pool_retains_only_salted_key_ids_not_raw_secrets() {
    let scope = "key-pool-test-no-secrets";
    let configured = vec!["sensitive_example_token".to_owned()];
    next_key(scope, &configured);
    let pools = registry().lock().unwrap();
    let key = &pools.get(scope).unwrap().keys[0];
    assert_eq!(key.id, secret_id("sensitive_example_token"));
    assert_ne!(key.id, secret_id("different_token"));
}
#[test]
fn deleted_model_scope_does_not_remove_sibling_key_health() {
    let prefix = "key-pool-test-model-prune:";
    let a = format!("{prefix}model-a");
    let b = format!("{prefix}model-b");
    next_key(&a, &["a".to_owned()]);
    next_key(&b, &["b".to_owned()]);
    forget(&a);
    let remaining = provider_snapshots(prefix);
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].0, "model-b");
    forget_provider(prefix);
}
#[test]
fn deleted_provider_scopes_are_pruned() {
    let prefix = "key-pool-test-prune:";
    next_key("key-pool-test-prune:model-a", &["a".to_owned()]);
    next_key("key-pool-test-prune:model-b", &["b".to_owned()]);
    forget_provider(prefix);
    assert!(provider_snapshots(prefix).is_empty());
}
#[test]
fn leases_are_released_without_underflow() {
    let scope = "key-pool-test-lease";
    let keys = vec!["a".to_owned()];
    next_key(scope, &keys);
    acquire(scope, "a");
    assert_eq!(snapshot(scope)[0].active_leases, 1);
    release(scope, "a");
    release(scope, "a");
    assert_eq!(snapshot(scope)[0].active_leases, 0);
}
