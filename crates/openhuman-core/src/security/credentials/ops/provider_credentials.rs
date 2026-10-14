//! Storing, removing, and listing provider (non-app-session) credentials.

use serde_json::json;

use crate::config::Config;
use crate::core::Outcome;
use crate::security::credentials::session_support::{
    parse_fields_value, profile_name_or_default, summarize_auth_profile,
};
use crate::security::credentials::{normalize_provider, AuthService, APP_SESSION_PROVIDER};

pub async fn store_provider_credentials(
    config: &Config,
    provider: &str,
    profile: Option<&str>,
    token: Option<String>,
    fields: Option<serde_json::Value>,
    set_active: Option<bool>,
) -> Result<Outcome<super::super::responses::AuthProfileSummary>, String> {
    let provider = provider.trim().to_string();
    if provider.is_empty() {
        return Err("provider is required".to_string());
    }

    let profile_name = profile_name_or_default(profile);
    let mut metadata = parse_fields_value(fields)?;
    let token = token
        .as_ref()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .or_else(|| metadata.get("token").cloned())
        .or_else(|| metadata.get("api_key").cloned())
        .unwrap_or_default();
    if token.is_empty() && metadata.is_empty() {
        return Err("provide at least one credential via token or fields".to_string());
    }
    metadata.remove("token");

    let auth = AuthService::from_config(config);
    let stored = auth
        .store_provider_token(
            &provider,
            profile_name,
            &token,
            metadata,
            set_active.unwrap_or(true),
        )
        .map_err(|e| e.to_string())?;
    // A freshly-stored key supersedes any prior auth rejection for this
    // provider — clear the recorded BYO auth error so the AI-settings notice
    // disappears and the notification latch re-arms (a future rejection will
    // notify again). Credentials are keyed `provider:<slug>`; the auth-error
    // registry is keyed by the bare provider slug used by the chat factory.
    clear_provider_auth_error(&provider);
    let mut summary = summarize_auth_profile(&stored);
    summary.is_active = set_active.unwrap_or(true);
    Ok(Outcome::single_log(summary, "provider credentials stored"))
}

/// Clear any recorded BYO provider auth error for a credentials `provider`
/// key. Strips the `provider:` namespace prefix so the lookup matches the
/// bare slug (`openrouter`) the inference classifier records under.
fn clear_provider_auth_error(provider: &str) {
    let slug = provider.strip_prefix("provider:").unwrap_or(provider);
    crate::inference::auth_error_registry::clear(slug);
}

pub async fn remove_provider_credentials(
    config: &Config,
    provider: &str,
    profile: Option<&str>,
) -> Result<Outcome<serde_json::Value>, String> {
    let profile_name = profile_name_or_default(profile);
    let auth = AuthService::from_config(config);
    let removed = auth
        .remove_profile(provider, profile_name)
        .map_err(|e| e.to_string())?;
    // Removing the key clears any recorded BYO auth error for this provider —
    // there is no longer a key to be "rejected", so the stale notice must go.
    clear_provider_auth_error(provider);
    Ok(Outcome::single_log(
        json!({
            "removed": removed,
            "provider": provider,
            "profile": profile_name,
        }),
        "provider credentials removed",
    ))
}

/// Switch a named provider account without copying or exposing its token.
/// This is also the explicit Codex OAuth account-selection RPC.
pub async fn activate_provider_credentials(
    config: &Config,
    provider: &str,
    profile: &str,
) -> Result<Outcome<super::super::responses::AuthProfileSummary>, String> {
    let provider = normalize_provider(provider).map_err(|_| "Invalid provider".to_string())?;
    if provider == APP_SESSION_PROVIDER {
        return Err("App session credentials cannot be selected as a provider account".into());
    }
    if profile.trim().is_empty() {
        return Err("Profile name is required".into());
    }
    let auth = AuthService::from_config(config);
    let selected = auth
        .set_active_profile(&provider, profile)
        .map_err(|_| "Requested provider account was not found".to_string())?;
    let stored = auth
        .get_profile(&provider, Some(&selected))
        .map_err(|_| "Could not load selected provider account".to_string())?
        .ok_or_else(|| "Selected provider account was not found".to_string())?;
    clear_provider_auth_error(&provider);
    let mut summary = summarize_auth_profile(&stored);
    summary.is_active = true;
    Ok(Outcome::single_log(
        summary,
        "active provider account changed",
    ))
}

pub async fn list_provider_credentials(
    config: &Config,
    provider_filter: Option<String>,
) -> Result<Outcome<Vec<super::super::responses::AuthProfileSummary>>, String> {
    let auth = AuthService::from_config(config);
    let provider_filter = provider_filter
        .map(|provider| normalize_provider(&provider))
        .transpose()
        .map_err(|e| e.to_string())?;
    let profiles = auth.load_profiles().map_err(|e| e.to_string())?;
    let mut items = profiles
        .profiles
        .values()
        .filter(|profile| profile.provider != APP_SESSION_PROVIDER)
        .filter(|profile| {
            provider_filter
                .as_ref()
                .is_none_or(|provider| profile.provider == *provider)
        })
        .map(|profile| {
            let mut summary = summarize_auth_profile(profile);
            summary.is_active = profiles
                .active_profiles
                .get(&profile.provider)
                .is_some_and(|id| id == &profile.id);
            summary
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.profile_name.cmp(&b.profile_name))
    });

    Ok(Outcome::single_log(items, "provider credentials listed"))
}

/// List credentials whose provider key starts with `prefix`.
///
/// Pure prefix variant of [`list_provider_credentials`] for namespaces
/// that group multiple providers under a common stem (e.g.
/// `"channel:"` covers `channel:telegram:managed_dm`,
/// `channel:slack:bot_token`, …). The exact-match filter on
/// `list_provider_credentials` cannot express this without enumerating
/// every concrete provider key up front.
pub async fn list_provider_credentials_by_prefix(
    config: &Config,
    prefix: &str,
) -> Result<Vec<super::super::responses::AuthProfileSummary>, String> {
    let auth = AuthService::from_config(config);
    let profiles = auth.load_profiles().map_err(|e| e.to_string())?;
    let mut items = profiles
        .profiles
        .values()
        .filter(|profile| profile.provider != APP_SESSION_PROVIDER)
        .filter(|profile| profile.provider.starts_with(prefix))
        .map(|profile| {
            let mut summary = summarize_auth_profile(profile);
            summary.is_active = profiles
                .active_profiles
                .get(&profile.provider)
                .is_some_and(|id| id == &profile.id);
            summary
        })
        .collect::<Vec<_>>();
    items.sort_by(|a, b| {
        a.provider
            .cmp(&b.provider)
            .then_with(|| a.profile_name.cmp(&b.profile_name))
    });
    Ok(items)
}

#[cfg(test)]
#[path = "provider_credentials_tests.rs"]
mod activation_tests;
