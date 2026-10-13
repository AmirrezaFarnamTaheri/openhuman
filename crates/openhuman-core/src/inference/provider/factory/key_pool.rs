//! Multi-key routing ported from Penguin's ApiKeyRotator.
//!
//! Credentials are stored per provider in the existing encrypted AuthService,
//! never in cloud provider metadata. This module does not log raw keys.
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const COOLDOWN: Duration = Duration::from_secs(60);

#[derive(Clone)]
struct KeyHealth {
    value: String,
    cooldown_until: Option<Instant>,
    evicted: bool,
    successes: u64,
    failures: u64,
}
impl KeyHealth {
    fn new(value: String) -> Self {
        Self { value, cooldown_until: None, evicted: false, successes: 0, failures: 0 }
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
#[allow(dead_code)] // Reserved for request-outcome hooks in the full fleet port.
pub enum KeyFailure {
    Authentication,
    RateLimited,
    Other,
}

fn registry() -> &'static Mutex<HashMap<String, Pool>> {
    static POOLS: OnceLock<Mutex<HashMap<String, Pool>>> = OnceLock::new();
    POOLS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Deduplicate comma/semicolon/newline-delimited secrets while preserving order.
pub fn parse_api_keys(raw: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for part in raw.split([',', ';', '\n', '\r']) {
        let key = part.trim();
        if !key.is_empty() && !keys.iter().any(|existing| existing == key) {
            keys.push(key.to_owned());
        }
    }
    keys
}

fn sync(pool: &mut Pool, configured: &[String]) {
    if pool.keys.len() == configured.len()
        && pool.keys.iter().zip(configured).all(|(state, key)| state.value == *key)
    {
        return;
    }
    let mut previous = std::mem::take(&mut pool.keys);
    pool.keys = configured.iter().map(|key| {
        if let Some(pos) = previous.iter().position(|state| state.value == *key) {
            previous.remove(pos)
        } else {
            KeyHealth::new(key.clone())
        }
    }).collect();
    pool.next = 0;
}

/// Choose only an eligible key. Never use a key during cooldown or after a 401.
pub fn next_key(scope: &str, configured: &[String]) -> Option<(usize, String)> {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let pool = pools.entry(scope.to_owned()).or_default();
    sync(pool, configured);
    let now = Instant::now();
    for offset in 0..pool.keys.len() {
        let index = (pool.next + offset) % pool.keys.len();
        if pool.keys[index].available(now) {
            pool.next = (index + 1) % pool.keys.len();
            return Some((index, pool.keys[index].value.clone()));
        }
    }
    None
}

/// Update shared health on a response, without retaining response bodies or tokens in logs.
#[allow(dead_code)] // Called by fleet tests; runtime outcome hooks are pending.
pub fn record_result(scope: &str, key_index: usize, failure: Option<KeyFailure>) {
    let mut pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(key) = pools.get_mut(scope).and_then(|pool| pool.keys.get_mut(key_index)) else { return };
    match failure {
        None => { key.successes += 1; key.cooldown_until = None; }
        Some(KeyFailure::Authentication) => { key.failures += 1; key.evicted = true; }
        Some(KeyFailure::RateLimited) => {
            key.failures += 1;
            key.cooldown_until = Some(Instant::now() + COOLDOWN);
        }
        Some(KeyFailure::Other) => { key.failures += 1; }
    }
}

/// Scrubbed health snapshot; never exposes tokens.
#[allow(dead_code)] // Read-only fleet health API for the planned UI.
pub fn health(scope: &str) -> Vec<(bool, bool, u64, u64)> {
    let pools = registry().lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    pools.get(scope).map(|pool| pool.keys.iter().map(|key| {
        (key.evicted, key.cooldown_until.is_some_and(|until| until > Instant::now()), key.successes, key.failures)
    }).collect()).unwrap_or_default()
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
        let keys = vec!["a".to_string(), "b".to_string()];
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
}
