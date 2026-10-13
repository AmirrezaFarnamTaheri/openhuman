//! Runtime credential pool shared by native BYOK model instances.
//!
//! Parsing mirrors Penguin's multi-key support; selection and outcomes are
//! maintained in Rust, where requests actually execute. This module never
//! exposes or logs secrets. Pool identity includes the local workspace.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const DEFAULT_COOLDOWN: Duration = Duration::from_secs(60);
const MAX_COOLDOWN: Duration = Duration::from_secs(300);

#[derive(Clone)]
struct KeyHealth {
    value: String,
    cooldown_until: Option<Instant>,
    evicted: bool,
    successes: u64,
    failures: u64,
    active: usize,
    last_used: Option<Instant>,
}
impl KeyHealth {
    fn new(value: String) -> Self {
        Self {
            value, cooldown_until: None, evicted: false,
            successes: 0, failures: 0, active: 0, last_used: None,
        }
    }
    fn available(&self, now: Instant) -> bool {
        !self.evicted && self.cooldown_until.is_none_or(|until| until <= now)
    }
}

#[derive(Default)]
struct Pool {
    keys: Vec<KeyHealth>,
    next: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyFailure {
    Authentication,
    RateLimited,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RotationStrategy {
    #[default]
    RoundRobin,
    LeastBusy,
    PriorityWeighted,
}
impl RotationStrategy {
    pub fn parse(raw: &str) -> Self {
        match raw {
            "least_busy" | "least-leases" => Self::LeastBusy,
            "priority_weighted" | "priority-weighted" => Self::PriorityWeighted,
            _ => Self::RoundRobin,
        }
    }
}

fn registry() -> &'static Mutex<HashMap<String, Pool>> {
    static POOLS: OnceLock<Mutex<HashMap<String, Pool>>> = OnceLock::new();
    POOLS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Read a secret group from comma, semicolon, or newline-separated credentials.
/// Empty segments are discarded and identical values are kept once.
pub fn parse_api_keys(raw: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for segment in raw.split([',', ';', '\n', '\r']) {
        let value = segment.trim();
        if !value.is_empty() && !keys.iter().any(|key| key == value) {
            keys.push(value.to_owned());
        }
    }
    keys
}

fn sync(pool: &mut Pool, configured: &[String]) {
    if pool.keys.len() == configured.len()
        && pool.keys.iter().zip(configured).all(|(entry, value)| entry.value == *value)
    {
        return;
    }
    let mut previous = std::mem::take(&mut pool.keys);
    pool.keys = configured.iter().map(|value| {
        match previous.iter().position(|entry| entry.value == *value) {
            Some(index) => previous.remove(index),
            None => KeyHealth::new(value.clone()),
        }
    }).collect();
    pool.next = 0;
}

/// Select without returning a still-cooling or invalid key.
/// `excluded` avoids accidentally retrying the same key within one request.
pub fn choose_key(
    scope: &str,
    configured: &[String],
    excluded: &[usize],
    strategy: RotationStrategy,
) -> Option<(usize, String)> {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let pool = pools.entry(scope.to_owned()).or_default();
    sync(pool, configured);
    if pool.keys.is_empty() { return None; }
    let now = Instant::now();
    let candidates: Vec<usize> = (0..pool.keys.len())
        .filter(|i| !excluded.contains(i) && pool.keys[*i].available(now))
        .collect();
    let index = match strategy {
        RotationStrategy::RoundRobin => (0..pool.keys.len())
            .map(|i| (pool.next + i) % pool.keys.len())
            .find(|i| candidates.contains(i)),
        RotationStrategy::LeastBusy => candidates.into_iter().min_by_key(|i| {
            (pool.keys[*i].active, pool.keys[*i].failures, pool.keys[*i].last_used)
        }),
        RotationStrategy::PriorityWeighted => candidates.into_iter().min_by_key(|i| {
            (pool.keys[*i].failures.saturating_mul(2).saturating_sub(pool.keys[*i].successes), pool.keys[*i].active)
        }),
    }?;
    pool.next = (index + 1) % pool.keys.len();
    pool.keys[index].last_used = Some(now);
    Some((index, pool.keys[index].value.clone()))
}

pub fn next_key_excluding(scope: &str, configured: &[String], excluded: &[usize]) -> Option<(usize, String)> {
    choose_key(scope, configured, excluded, RotationStrategy::RoundRobin)
}
pub fn next_key(scope: &str, configured: &[String]) -> Option<(usize, String)> {
    next_key_excluding(scope, configured, &[])
}

/// A lease is held through a full unary request or a stream's lifetime.
pub fn acquire(scope: &str, key: &str) {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(entry) = pools.get_mut(scope).and_then(|pool| pool.keys.iter_mut().find(|k| k.value == key)) {
        entry.active += 1;
    }
}
pub fn release(scope: &str, key: &str) {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(entry) = pools.get_mut(scope).and_then(|pool| pool.keys.iter_mut().find(|k| k.value == key)) {
        entry.active = entry.active.saturating_sub(1);
    }
}

/// Attribute outcomes to secret identity, not a positional index that may have
/// changed after a settings update while the request was running.
pub fn record_result_for(scope: &str, key: &str, failure: Option<KeyFailure>, retry_after_ms: Option<u64>) {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(entry) = pools.get_mut(scope).and_then(|pool| pool.keys.iter_mut().find(|item| item.value == key)) else { return };
    match failure {
        None => { entry.successes += 1; entry.cooldown_until = None; }
        Some(KeyFailure::Authentication) => { entry.failures += 1; entry.evicted = true; }
        Some(KeyFailure::RateLimited) => {
            entry.failures += 1;
            let delay = Duration::from_millis(retry_after_ms.unwrap_or(DEFAULT_COOLDOWN.as_millis() as u64))
                .min(MAX_COOLDOWN);
            entry.cooldown_until = Some(Instant::now() + delay);
        }
        Some(KeyFailure::Other) => { entry.failures += 1; }
    }
}
pub fn record_result(scope: &str, key_index: usize, failure: Option<KeyFailure>) {
    let value = {
        let pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        pools.get(scope).and_then(|pool| pool.keys.get(key_index)).map(|key| key.value.clone())
    };
    if let Some(value) = value { record_result_for(scope, &value, failure, None); }
}

/// Safe summary used by settings/diagnostics; deliberately no masked or raw keys.
#[derive(Clone, Debug, serde::Serialize)]
pub struct KeyStats {
    pub index: usize,
    pub status: &'static str,
    pub successes: u64,
    pub failures: u64,
    pub active_leases: usize,
    pub cooldown_remaining_ms: u64,
}
pub fn snapshot(scope: &str) -> Vec<KeyStats> {
    let pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let now = Instant::now();
    pools.get(scope).map(|pool| pool.keys.iter().enumerate().map(|(i, key)| {
        let cooldown = key.cooldown_until.and_then(|until| until.checked_duration_since(now))
            .unwrap_or_default().as_millis() as u64;
        KeyStats {
            index: i,
            status: if key.evicted { "evicted" } else if cooldown > 0 { "cooldown" } else { "healthy" },
            successes: key.successes,
            failures: key.failures,
            active_leases: key.active,
            cooldown_remaining_ms: cooldown,
        }
    }).collect()).unwrap_or_default()
}
pub fn health(scope: &str) -> Vec<(bool, bool, u64, u64)> {
    snapshot(scope).iter().map(|s| (s.status == "evicted", s.status == "cooldown", s.successes, s.failures)).collect()
}
pub fn reset(scope: &str) {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(pool) = pools.get_mut(scope) {
        for entry in &mut pool.keys {
            entry.evicted = false;
            entry.cooldown_until = None;
            entry.successes = 0;
            entry.failures = 0;
        }
    }
}

/// Health views for all currently initialized models in a provider group.
/// The model name is derived from the scope; no token identifier is exposed.
pub fn provider_snapshots(prefix: &str) -> Vec<(String, Vec<KeyStats>)> {
    let scopes: Vec<String> = {
        let pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        pools.keys().filter(|name| name.starts_with(prefix)).cloned().collect()
    };
    let mut snapshots: Vec<_> = scopes.into_iter().map(|scope| {
        let model = scope.strip_prefix(prefix).unwrap_or_default().to_owned();
        (model, snapshot(&scope))
    }).collect();
    snapshots.sort_by(|a, b| a.0.cmp(&b.0));
    snapshots
}

/// Clearing a provider's state revives evicted keys without changing secrets.
pub fn reset_provider(prefix: &str) {
    let scopes: Vec<String> = {
        let pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        pools.keys().filter(|name| name.starts_with(prefix)).cloned().collect()
    };
    for scope in scopes { reset(&scope); }
}

/// Called when credentials are replaced, to erase stale secrets and state.
pub fn forget(scope: &str) {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    pools.remove(scope);
}

#[cfg(test)]
mod tests {
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
}
