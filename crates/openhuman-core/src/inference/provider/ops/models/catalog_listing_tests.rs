use super::*;

#[test]
fn retries_only_auth_rejections_while_untried_credentials_remain() {
    assert!(should_try_next_catalog_key(401, true));
    assert!(should_try_next_catalog_key(403, true));
    assert!(!should_try_next_catalog_key(401, false));
    assert!(!should_try_next_catalog_key(408, true));
    assert!(!should_try_next_catalog_key(429, true));
    assert!(!should_try_next_catalog_key(500, true));
}

#[test]
fn openrouter_validation_tries_another_key_only_for_auth_status() {
    assert!(should_retry_openrouter_key(
        "OpenRouter key validation returned 401: invalid API key", true
    ));
    assert!(should_retry_openrouter_key(
        "OpenRouter key validation returned 403: forbidden", true
    ));
    assert!(!should_retry_openrouter_key(
        "OpenRouter key validation returned 401: invalid API key", false
    ));
    assert!(!should_retry_openrouter_key(
        "OpenRouter key validation failed: connection refused", true
    ));
}
