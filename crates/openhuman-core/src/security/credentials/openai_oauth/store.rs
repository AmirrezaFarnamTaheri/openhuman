//! Persist and resolve OpenAI OAuth tokens for the `openai` cloud provider slug.

use std::path::{Path, PathBuf};

use chrono::{Duration, TimeZone, Utc};
use sha2::{Digest, Sha256};
use tinyinference_providers::oauth::{
    openai_account_id_from_access_token, openai_codex_config, parse_openai_codex_auth_json,
    refresh_access_token, OAuthTokenSet,
};

use crate::config::Config;
use crate::security::credentials::profiles::{AuthProfile, AuthProfilesStore, TokenSet};
use crate::security::credentials::{state_dir_from_config, AuthService};

const LOG_PREFIX: &str = "[inference][openai-oauth][store]";

pub const OPENAI_PROVIDER_KEY: &str = "provider:openai";
pub const OPENAI_OAUTH_PROFILE_NAME: &str = "oauth";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiOAuthCredentials {
    pub access_token: String,
    pub account_id: Option<String>,
}

fn token_set_from_oauth(token: &OAuthTokenSet) -> TokenSet {
    let expires_at =
        (token.expires_in > 0).then(|| Utc::now() + Duration::seconds(token.expires_in as i64));
    TokenSet {
        access_token: token.access_token.clone(),
        refresh_token: token
            .refresh_token
            .clone()
            .filter(|refresh| !refresh.is_empty()),
        id_token: token.id_token.clone(),
        expires_at,
        token_type: Some("Bearer".to_string()),
        scope: None,
    }
}

fn normalize_persisted_token_set(mut token_set: TokenSet) -> Result<TokenSet, String> {
    let access_token = token_set.access_token.trim().to_string();
    if access_token.is_empty() {
        return Err("OpenAI OAuth token is missing access_token".to_string());
    }
    token_set.access_token = access_token;
    token_set.refresh_token = token_set.refresh_token.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    token_set.id_token = token_set.id_token.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    token_set.token_type = token_set.token_type.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    token_set.scope = token_set.scope.and_then(|value| {
        let trimmed = value.trim().to_string();
        (!trimmed.is_empty()).then_some(trimmed)
    });
    Ok(token_set)
}

pub fn persist_openai_oauth_token(
    config: &Config,
    token: &OAuthTokenSet,
) -> Result<AuthProfile, String> {
    let account_id = openai_account_id_from_access_token(token.access_token.trim());
    persist_openai_oauth_token_set(config, token_set_from_oauth(token), account_id)
}

/// Each authorized ChatGPT account owns its own encrypted OAuth profile.
/// The first account retains the legacy "oauth" name; subsequent logins
/// get stable, non-reversible account-id-derived names. Re-login reuses the
/// existing account instead of creating an unbounded stack of tokens.
pub(super) fn persist_openai_oauth_token_set(
    config: &Config,
    token_set: TokenSet,
    account_id: Option<String>,
) -> Result<AuthProfile, String> {
    use crate::security::credentials::profiles::AuthProfileKind;

    let token_set = normalize_persisted_token_set(token_set)?;
    let account_id = account_id
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    let store = auth_profiles_store(config);
    let existing = store
        .load()
        .map_err(|_| "Could not read Codex account profiles".to_string())?;
    let oauth_profiles: Vec<&AuthProfile> = existing
        .profiles
        .values()
        .filter(|p| p.provider == OPENAI_PROVIDER_KEY && p.kind == AuthProfileKind::OAuth)
        .collect();

    let name = if let Some(ref account_id) = account_id {
        oauth_profiles
            .iter()
            .find(|p| p.metadata.get("account_id") == Some(account_id))
            .map(|p| p.profile_name.clone())
            .unwrap_or_else(|| {
                if oauth_profiles.is_empty() {
                    OPENAI_OAUTH_PROFILE_NAME.to_string()
                } else {
                    let digest = Sha256::digest(account_id.as_bytes());
                    format!("oauth-{}", &hex::encode(digest)[..16])
                }
            })
    } else if oauth_profiles.is_empty() {
        OPENAI_OAUTH_PROFILE_NAME.to_string()
    } else {
        // Without an upstream account identifier we cannot safely establish
        // that this login belongs to an existing account; silently replacing
        // its refresh token would destroy the first user's session.
        return Err(
            "Codex account identity is missing; reconnect with an identifiable account".into(),
        );
    };

    let mut profile = AuthProfile::new_oauth(OPENAI_PROVIDER_KEY, &name, token_set);
    if let Some(account_id) = account_id {
        profile
            .metadata
            .insert("account_id".to_string(), account_id);
    }
    store
        .upsert_profile(profile.clone(), true)
        .map_err(|e| e.to_string())?;
    Ok(profile)
}

/// Select the currently active Codex OAuth account; retain the historical
/// "oauth" fallback for workspaces with a preferred API-key profile.
pub(super) fn selected_openai_oauth_profile(
    config: &Config,
) -> Result<Option<AuthProfile>, String> {
    use crate::security::credentials::profiles::AuthProfileKind;
    let auth = AuthService::from_config(config);
    let active = auth
        .get_profile(OPENAI_PROVIDER_KEY, None)
        .map_err(|e| e.to_string())?;
    if active
        .as_ref()
        .is_some_and(|p| p.kind == AuthProfileKind::OAuth)
    {
        return Ok(active);
    }
    if let Some(legacy) = auth
        .get_profile(OPENAI_PROVIDER_KEY, Some(OPENAI_OAUTH_PROFILE_NAME))
        .map_err(|e| e.to_string())?
        .filter(|p| p.kind == AuthProfileKind::OAuth)
    {
        return Ok(Some(legacy));
    }
    let data = auth.load_profiles().map_err(|e| e.to_string())?;
    Ok(data
        .profiles
        .values()
        .find(|profile| {
            profile.provider == OPENAI_PROVIDER_KEY && profile.kind == AuthProfileKind::OAuth
        })
        .cloned())
}

fn codex_cli_auth_path() -> Result<PathBuf, String> {
    if let Some(codex_home) = std::env::var_os("CODEX_HOME") {
        let path = PathBuf::from(codex_home);
        if !path.as_os_str().is_empty() {
            return Ok(path.join("auth.json"));
        }
    }

    let home = home_dir_from_env()
        .ok_or_else(|| "home directory is not set; cannot find ~/.codex/auth.json".to_string())?;
    Ok(home.join(".codex").join("auth.json"))
}

fn home_dir_from_env() -> Option<PathBuf> {
    for key in ["HOME", "USERPROFILE"] {
        if let Some(value) = std::env::var_os(key) {
            let path = PathBuf::from(value);
            if !path.as_os_str().is_empty() {
                return Some(path);
            }
        }
    }

    match (std::env::var_os("HOMEDRIVE"), std::env::var_os("HOMEPATH")) {
        (Some(drive), Some(path))
            if !drive.as_os_str().is_empty() && !path.as_os_str().is_empty() =>
        {
            Some(PathBuf::from(drive).join(path))
        }
        _ => None,
    }
}

pub(super) fn import_codex_cli_auth_from_path(
    config: &Config,
    path: &Path,
) -> Result<AuthProfile, String> {
    log::info!(
        "{LOG_PREFIX} codex_cli_import:start path={}",
        path.display()
    );
    let bytes = std::fs::read(path).map_err(|e| {
        log::warn!(
            "{LOG_PREFIX} codex_cli_import:read_failed path={} error={e}",
            path.display()
        );
        format!(
            "Could not read Codex CLI auth at {}: {e}. Run `codex login` first, then try Codex auth again.",
            path.display()
        )
    })?;
    let imported = parse_openai_codex_auth_json(&bytes).map_err(|e| {
        log::warn!(
            "{LOG_PREFIX} codex_cli_import:parse_failed path={} error={e}",
            path.display()
        );
        format!(
            "Could not parse Codex CLI auth at {}: {e}. Run `codex login` again, then try Codex auth again.",
            path.display()
        )
    })?;
    let account_id = imported.account_id;

    log::info!(
        "{LOG_PREFIX} codex_cli_import:persist_start path={} account_id_present={}",
        path.display(),
        account_id.is_some()
    );
    let profile = persist_openai_oauth_token_set(
        config,
        TokenSet {
            access_token: imported.token.access_token,
            refresh_token: imported.token.refresh_token,
            id_token: imported.token.id_token,
            expires_at: imported
                .expires_at_unix
                .and_then(|timestamp| Utc.timestamp_opt(timestamp, 0).single()),
            token_type: Some("Bearer".to_string()),
            scope: None,
        },
        account_id,
    )?;
    log::info!(
        "{LOG_PREFIX} codex_cli_import:ok path={} profile_id={}",
        path.display(),
        profile.id
    );
    Ok(profile)
}

pub fn import_codex_cli_auth(config: &Config) -> Result<AuthProfile, String> {
    let path = codex_cli_auth_path()?;
    import_codex_cli_auth_from_path(config, &path)
}

fn auth_profiles_store(config: &Config) -> AuthProfilesStore {
    AuthProfilesStore::new(&state_dir_from_config(config), config.secrets.encrypt)
}

fn try_refresh_oauth_token(refresh: &str) -> Result<OAuthTokenSet, String> {
    let cfg = openai_codex_config("http://127.0.0.1:1455/auth/callback");
    let refresh = refresh.to_string();
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        // `block_in_place` lets the multi-thread runtime move other tasks off this
        // worker before we synchronously drive the refresh future, avoiding a
        // deadlock when this lookup is reached from inside an async caller.
        return tokio::task::block_in_place(|| {
            handle.block_on(refresh_access_token(&cfg, &refresh))
        })
        .map_err(|e| e.to_string());
    }
    Err("tokio runtime required to refresh openai oauth token".to_string())
}

/// Look up the OpenAI bearer token sourced from the OAuth (ChatGPT
/// subscription) flow. Returns `Ok(None)` when no OAuth profile is present or
/// when the access token is empty. API-key fallback for the `openai` slug is
/// handled by the standard `lookup_key_for_slug` path — this function is
/// OAuth-only so the standard path's env/audit/metrics logic still runs.
pub fn lookup_openai_bearer_token(config: &Config) -> Result<Option<String>, String> {
    Ok(lookup_openai_oauth_credentials(config)?.map(|credentials| credentials.access_token))
}

fn credentials_from_oauth_profile(
    config: &Config,
    mut profile: AuthProfile,
) -> Result<Option<OpenAiOAuthCredentials>, String> {
    let Some(mut token_set) = profile.token_set.clone() else {
        return Ok(None);
    };
    let skew = Duration::minutes(2);
    if token_set.is_expiring_within(std::time::Duration::from_secs(
        skew.num_seconds().unsigned_abs(),
    )) {
        if let Some(refresh) = token_set.refresh_token.clone() {
            match try_refresh_oauth_token(&refresh) {
                Ok(fresh) => {
                    token_set = token_set_from_oauth(&fresh);
                    profile.token_set = Some(token_set.clone());
                    // Refreshing a standby account must NEVER hijack the
                    // user's selected account. Existing implementations used
                    // set_active=true here, which was safe for only one login.
                    let selected = AuthService::from_config(config)
                        .get_profile(OPENAI_PROVIDER_KEY, None)
                        .map_err(|_| "Codex account selection unavailable".to_string())?;
                    let keep_active = selected.as_ref().is_some_and(|p| p.id == profile.id);
                    if auth_profiles_store(config)
                        .upsert_profile(profile.clone(), keep_active)
                        .is_err()
                    {
                        log::warn!("{LOG_PREFIX} refreshed Codex token was not persisted");
                    }
                }
                Err(err) => {
                    log::warn!("{LOG_PREFIX} OAuth token refresh failed");
                    if token_set.is_expiring_within(std::time::Duration::ZERO) {
                        return Err(format!(
                            "Codex authentication token is expired — refresh failed: {err}. \
                             Please reconnect Codex in Settings → Integrations."
                        ));
                    }
                }
            }
        }
    }
    let access = token_set.access_token.trim();
    if access.is_empty() {
        return Ok(None);
    }
    Ok(Some(OpenAiOAuthCredentials {
        access_token: access.to_string(),
        account_id: profile
            .metadata
            .get("account_id")
            .map(String::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string)
            .or_else(|| openai_account_id_from_access_token(access)),
    }))
}

pub fn lookup_openai_oauth_credentials(
    config: &Config,
) -> Result<Option<OpenAiOAuthCredentials>, String> {
    match selected_openai_oauth_profile(config)? {
        Some(profile) => credentials_from_oauth_profile(config, profile),
        None => Ok(None),
    }
}

/// Materialize independent authorized Codex identities with matching
/// bearer/account-id pairs. The user's active account leads the fleet,
/// while standby accounts rotate only after each token is independently
/// refreshed and validated for non-blank bearer state.
/// OAuth accounts never mix with API-key profiles.
pub fn lookup_openai_oauth_fleet(config: &Config) -> Result<Vec<OpenAiOAuthCredentials>, String> {
    use crate::security::credentials::profiles::AuthProfileKind;
    let auth = AuthService::from_config(config);
    let selected = selected_openai_oauth_profile(config)?.map(|p| p.id);
    let snapshot = auth
        .load_profiles()
        .map_err(|_| "Codex account list is unavailable".to_string())?;
    let mut profiles: Vec<AuthProfile> = snapshot
        .profiles
        .values()
        .filter(|p| p.provider == OPENAI_PROVIDER_KEY && p.kind == AuthProfileKind::OAuth)
        .cloned()
        .collect();
    profiles.sort_by(|a, b| {
        (selected.as_ref() != Some(&a.id))
            .cmp(&(selected.as_ref() != Some(&b.id)))
            .then_with(|| a.profile_name.cmp(&b.profile_name))
    });
    let mut accounts = Vec::new();
    let mut first_failure = None;
    for profile in profiles {
        match credentials_from_oauth_profile(config, profile) {
            Ok(Some(account))
                if !accounts
                    .iter()
                    .any(|p: &OpenAiOAuthCredentials| p.access_token == account.access_token) =>
            {
                accounts.push(account);
            }
            Ok(_) => {}
            Err(error) => {
                if first_failure.is_none() {
                    first_failure = Some(error);
                }
            }
        }
    }
    if accounts.is_empty() {
        if let Some(error) = first_failure {
            return Err(error);
        }
    }
    Ok(accounts)
}
