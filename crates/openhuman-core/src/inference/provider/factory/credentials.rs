//! Auth-profile credential lookup for `<slug>` cloud providers, including the
//! OpenAI OAuth fallback and the scoped legacy `config.api_key` fallback.

use super::*;

/// Auth-profile storage key for a slug-keyed provider.
///
/// New writes use `"provider:<slug>"`. Lookups also try the bare `<slug>`
/// as a legacy fallback (old configs stored keys as e.g. `"openai:default"`).
pub fn auth_key_for_slug(slug: &str) -> String {
    format!("provider:{slug}")
}

/// Whether the openai bearer that [`lookup_key_for_slug`] resolves is an OAuth
/// (Codex-subscription) credential rather than a standard API key.
///
/// OAuth and API-key credentials share the same `provider:openai` profile store
/// and differ only by [`AuthProfileKind`], so the bearer *string* cannot reveal
/// its source — which is exactly why the old `access_token == bearer_key` compare
/// broke under token rotation (#5353). This mirrors `lookup_key_for_slug`'s
/// precedence (`provider:openai`, then the legacy bare `openai`) and reports the
/// *kind* of the profile that would win. With no stored openai profile carrying a
/// credential, the only bearer source is the OAuth fallback, so a present OAuth
/// credential means the bearer is OAuth.
pub(crate) fn openai_bearer_is_oauth(config: &Config) -> bool {
    use crate::security::credentials::profiles::AuthProfileKind;

    let auth = AuthService::from_config(config);
    for provider in [auth_key_for_slug("openai"), "openai".to_string()] {
        if let Ok(Some(profile)) = auth.get_profile(&provider, None) {
            // A profile with an empty credential is skipped by
            // `lookup_key_for_slug`, so fall through to the next precedence level.
            let has_credential = match profile.kind {
                AuthProfileKind::Token => profile
                    .token
                    .as_deref()
                    .is_some_and(|t| !t.trim().is_empty()),
                AuthProfileKind::OAuth => profile
                    .token_set
                    .as_ref()
                    .is_some_and(|t| !t.access_token.trim().is_empty()),
            };
            if has_credential {
                return matches!(profile.kind, AuthProfileKind::OAuth);
            }
        }
    }
    // No stored openai profile with a credential → the bearer, if any, comes from
    // the OAuth fallback (`lookup_openai_bearer_token`).
    crate::security::credentials::openai_oauth::lookup_openai_oauth_credentials(config)
        .ok()
        .flatten()
        .is_some()
}

/// Drop the underlying credential-store failure rather than exposing an error
/// chain which may contain raw access tokens, file contents, or bearer headers.
/// Only the independently authored, nonsecret diagnostic reaches callers.
pub(super) fn redact_credential_lookup_error(
    _cause: impl std::fmt::Display,
    safe_message: String,
) -> anyhow::Error {
    anyhow::anyhow!(safe_message)
}

/// Fetch the bearer token for a slug from the workspace `auth-profiles.json`.
///
/// Tries `provider:<slug>` first (new key format), then the bare `<slug>`
/// (legacy format where keys were stored as `"openai"`, `"anthropic"`, etc.).
/// Missing or empty keys return `Ok(String::new())` — callers treat that as
/// "no auth", which surfaces an authentication error at first call rather than
/// at factory build time.
pub fn lookup_key_for_slug(slug: &str, config: &Config) -> anyhow::Result<String> {
    // Ahead of the stored profiles, and scoped to the one slug the per-call
    // route registers. A caller that named an endpoint and a bearer for this
    // turn has said where the credential comes from, and there is nothing on
    // disk to find for a slug that exists only in this `Config` copy. Scoping it
    // by slug is what keeps the bearer from reaching a provider the caller never
    // named — the same containment the legacy `config.api_key` fallback below
    // gets from `legacy_inference_slug`.
    if slug == crate::config::schema::EPHEMERAL_ROUTE_SLUG {
        if let Some(route) = config.ephemeral_route.as_ref() {
            log::debug!(
                "[providers][chat-factory] auth lookup slug={} key_present={} (per-call route)",
                slug,
                !route.api_key.trim().is_empty()
            );
            return Ok(route.api_key.trim().to_string());
        }
    }

    let auth = AuthService::from_config(config);
    // Try new-style key first.
    let new_key = auth_key_for_slug(slug);
    if let Ok(Some(k)) = auth.get_provider_bearer_token(&new_key, None) {
        if !k.is_empty() {
            log::debug!(
                "[providers][chat-factory] auth lookup slug={} key_present=true (new-style)",
                slug
            );
            return Ok(k);
        }
    }
    // Fall back to legacy bare slug.
    let key = auth
        .get_provider_bearer_token(slug, None)
        .map_err(|source| {
            redact_credential_lookup_error(
                source,
                format!("[chat-factory] failed to read API key for slug '{slug}'"),
            )
        })?
        .unwrap_or_default();
    if !key.is_empty() {
        log::debug!(
            "[providers][chat-factory] auth lookup slug={} key_present=true",
            slug
        );
        return Ok(key);
    }

    // OAuth fallback for `openai` runs only after standard API-key resolution
    // returns empty, so env/audit/metrics in the standard path always execute
    // and the OAuth path never silently bypasses provider-agnostic logic.
    if slug == "openai" {
        match crate::security::credentials::openai_oauth::lookup_openai_bearer_token(config) {
            Ok(Some(token)) if !token.is_empty() => {
                log::debug!(
                    "[providers][chat-factory] auth lookup slug={} key_present=true (oauth)",
                    slug
                );
                return Ok(token);
            }
            Ok(_) => {}
            Err(source) => {
                return Err(redact_credential_lookup_error(
                    source,
                    "[chat-factory] openai oauth lookup failed".to_string(),
                ));
            }
        }
    }

    // Fallback: read from top-level config.api_key (direct config.toml api_key).
    // This handles the case where a key was set in config.toml but not saved
    // through the UI into auth-profiles.json.
    //
    // Scoped to the legacy direct-inference provider only — the cloud-provider
    // slug whose endpoint matches `config.inference_url`. `config.api_key` was
    // historically paired with `inference_url` for direct endpoint routing, so
    // an unscoped fallback would leak this global key to any other provider
    // whose auth-profile lookup returned empty (cross-provider credential leak
    // flagged by CodeRabbit + maintainers on #2724).
    if legacy_inference_slug(config) == Some(slug) {
        if let Some(config_key) = config.api_key.as_ref() {
            if !config_key.trim().is_empty() {
                log::debug!(
                    "[providers][chat-factory] auth lookup slug={} key_present=true (config.toml fallback for legacy inference_url)",
                    slug
                );
                return Ok(config_key.trim().to_string());
            }
        }
    }

    log::debug!(
        "[providers][chat-factory] auth lookup slug={} key_present=false",
        slug
    );
    Ok(String::new())
}

/// Include separately stored, *named* API-token profiles in this provider's
/// native per-request credential pool. Profile names and keys never cross
/// provider boundaries. An OAuth subscription is NOT an API key: it must
/// retain its own refreshed bearer and account-specific routing metadata.
///
/// The preferred credential keeps precedence for existing workspaces. Legacy
/// comma/newline-delimited groups continue to work unchanged.
pub(super) fn lookup_token_account_fleet(
    slug: &str,
    config: &Config,
    preferred: &str,
    using_oauth: bool,
) -> anyhow::Result<Vec<String>> {
    use crate::security::credentials::profiles::AuthProfileKind;
    let mut keys = super::key_pool::parse_api_keys(preferred);
    // Prodex has its own upstream account rotation; passing multiple virtual
    // keys breaks continuation affinity. Ephemeral routes must not inherit
    // disk-backed secrets, and Codex OAuth bearers need per-account headers.
    if using_oauth || slug == "prodex" || slug == crate::config::schema::EPHEMERAL_ROUTE_SLUG {
        return Ok(keys);
    }
    let expected = auth_key_for_slug(slug);
    let auth = AuthService::from_config(config);
    let data = auth.load_profiles().map_err(|_| {
        anyhow::anyhow!("[chat-factory] named provider credentials are unavailable")
    })?;
    for profile in data.profiles.values() {
        if profile.kind != AuthProfileKind::Token
            || (profile.provider != expected && profile.provider != slug)
        {
            continue;
        }
        if let Some(raw) = profile.token.as_deref() {
            for key in super::key_pool::parse_api_keys(raw) {
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }
        }
    }
    Ok(keys)
}

/// Return a safe-to-log representation of a URL endpoint: `scheme://host` only.
pub fn redact_endpoint(url: &str) -> String {
    let trimmed = url.trim();
    if let Some(rest) = trimmed.split_once("://") {
        let scheme = rest.0;
        let authority = rest.1.split('/').next().unwrap_or("");
        if authority.contains('@') {
            return "<redacted>".to_string();
        }
        let host = authority;
        let host_no_query = host.split('?').next().unwrap_or(host);
        return format!("{}://{}", scheme, host_no_query);
    }
    "<endpoint>".to_string()
}
