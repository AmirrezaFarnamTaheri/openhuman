//! The `list_configured_models` RPC entry point: resolves a provider id to a
//! `cloud_providers` entry (falling back to synthesized local-runtime /
//! managed entries), builds an authenticated `/models` request for it, and
//! parses the response.

use super::*;

/// Resolve the API key used to probe a provider's `/models` endpoint.
///
/// Cloud providers store their key in auth-profiles.json (via `lookup_key_for_slug`),
/// but the OMLX local-runtime chip persists its Bearer key in
/// `config.local_ai.api_key`. When the slug-scoped lookup comes back empty for omlx,
/// fall back to that local-runtime key so the reachability probe still sends
/// `Authorization: Bearer` (otherwise the OMLX server returns 401).
pub(super) fn resolve_local_runtime_key(
    slug: &str,
    looked_up: String,
    config: &crate::config::Config,
) -> String {
    let looked_up = looked_up.trim().to_string();
    if !looked_up.is_empty() {
        return looked_up;
    }
    if slug == "omlx" {
        return config
            .local_ai
            .api_key
            .clone()
            .unwrap_or_default()
            .trim()
            .to_string();
    }
    looked_up
}

/// Whether a provider entry can be probed at `{endpoint}/models`.
///
/// Claude Code is a local CLI provider, and a `cli://…` endpoint is a
/// placeholder rather than an API base. Any other scheme (`ftp://…`) is a
/// misconfiguration and is left to the request path to surface as an error.
/// An endpoint that does not parse as `scheme://host` at all is left to the
/// request path so a genuine misconfiguration still surfaces as an error.
fn endpoint_hosts_models_listing(slug: &str, endpoint: &str) -> bool {
    if slug == "claude-code" {
        return false;
    }
    match reqwest::Url::parse(endpoint.trim()) {
        Ok(url) if url.has_host() => url.scheme() != "cli",
        _ => true,
    }
}

/// Only authorization failures are informative about an individual
/// credential; network failures and server errors are not key-specific.
fn should_try_next_catalog_key(status: u16, has_more: bool) -> bool {
    has_more && matches!(status, 401 | 403)
}

/// A 200 response to /models is proof of catalogue reachability, NOT proof
/// of an accepted API key: some vendors intentionally expose a public model
/// catalogue. Check whether an otherwise identical request with no credential
/// is rejected. This is only an authentication-evidence probe; it cannot prove
/// billable inference access and must never send the candidate secret.
async fn catalogue_requires_credential(
    client: &reqwest::Client,
    models_url: &str,
    auth_style: AuthStyle,
) -> bool {
    let mut request = client.get(models_url);
    if auth_style == AuthStyle::Anthropic {
        request = request.header("anthropic-version", "2023-06-01");
    }
    matches!(
        request.send().await,
        Ok(response) if matches!(response.status().as_u16(), 401 | 403)
    )
}

/// OpenRouter's /key preflight returns a sanitized error string. Only
/// explicit HTTP auth failures permit trying another key.
fn should_retry_openrouter_key(error: &str, has_more: bool) -> bool {
    has_more
        && (error.starts_with("OpenRouter key validation returned 401:")
            || error.starts_with("OpenRouter key validation returned 403:"))
}

pub fn append_query_param(url: &str, key: &str, value: &str) -> String {
    if let Ok(mut parsed) = reqwest::Url::parse(url) {
        parsed.query_pairs_mut().append_pair(key, value);
        return parsed.to_string();
    }

    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}{key}={value}")
}

pub async fn list_configured_models(
    provider_id: &str,
) -> Result<crate::core::Outcome<serde_json::Value>, String> {
    let config = crate::config::Config::load_or_init()
        .await
        .map_err(|e| e.to_string())?;

    list_configured_models_from_config(provider_id, &config).await
}

/// Probe a candidate credential without altering the saved auth-profile store.
/// This is used by provider editing before replacing an encrypted account key.
/// Apply an uncommitted provider edit to the in-memory probe snapshot.
/// Neither the configured endpoints nor the credential store is changed.
fn apply_candidate_provider_preview(
    config: &mut crate::config::Config,
    provider_id: &str,
    entry: crate::config::schema::cloud_providers::CloudProviderCreds,
    ca_cert_pem: Option<String>,
) -> Result<(), String> {
    use crate::config::schema::cloud_providers::is_slug_reserved;
    if is_slug_reserved(&entry.slug) || entry.slug != provider_id {
        return Err("Candidate provider must match a non-reserved provider slug".into());
    }
    if entry.endpoint.len() > 8192 || entry.slug.len() > 128 {
        return Err("Candidate provider identifier or endpoint exceeds maximum length".into());
    }
    if !entry.id.is_empty() && config.cloud_providers.iter().any(|existing| {
        existing.id == entry.id && existing.slug != entry.slug
    }) {
        return Err("Candidate provider id conflicts with a different provider".into());
    }
    // Validate controls through the same Rust normalizers as saved settings.
    crate::config::schema::cloud_providers::normalize_fleet_config(
        entry.image_endpoint.clone(),
        entry.model_image_endpoints.clone(),
        entry.rotation_strategy.clone(),
    )?;
    crate::config::schema::cloud_providers::normalize_model_transports(
        entry.model_transports.clone(),
    )?;
    if let Some(pem) = ca_cert_pem {
        let checked = crate::config::rpc::collect_provider_ca_certs([
            (entry.slug.as_str(), Some(pem.as_str()))
        ])?;
        if pem.is_empty() {
            config.cloud_provider_ca_certs.remove(&entry.slug);
        } else if let Some(value) = checked.get(&entry.slug) {
            config.cloud_provider_ca_certs.insert(entry.slug.clone(), value.clone());
        }
    }
    config.cloud_providers.retain(|existing| existing.slug != entry.slug);
    config.cloud_providers.push(entry);
    Ok(())
}

pub async fn list_configured_models_with_candidate_key(
    provider_id: &str,
    candidate_key: &str,
    preview: Option<crate::config::schema::cloud_providers::CloudProviderCreds>,
    preview_ca_cert_pem: Option<String>,
) -> Result<crate::core::Outcome<serde_json::Value>, String> {
    let mut config = crate::config::Config::load_or_init()
        .await
        .map_err(|e| e.to_string())?;
    if let Some(entry) = preview {
        apply_candidate_provider_preview(
            &mut config, provider_id, entry, preview_ca_cert_pem
        )?;
    } else if preview_ca_cert_pem.is_some() {
        return Err("CA preview requires a candidate provider entry".into());
    }
    list_configured_models_with_candidate_key_from_config(provider_id, &config, candidate_key).await
}

pub async fn list_configured_models_from_config(
    provider_id: &str,
    config: &crate::config::Config,
) -> Result<crate::core::Outcome<serde_json::Value>, String> {
    list_configured_models_inner(provider_id, config, None).await
}

pub async fn list_configured_models_with_candidate_key_from_config(
    provider_id: &str,
    config: &crate::config::Config,
    candidate_key: &str,
) -> Result<crate::core::Outcome<serde_json::Value>, String> {
    list_configured_models_inner(provider_id, config, Some(candidate_key)).await
}

async fn list_configured_models_inner(
    provider_id: &str,
    config: &crate::config::Config,
    candidate_key: Option<&str>,
) -> Result<crate::core::Outcome<serde_json::Value>, String> {
    let provider_id = provider_id.trim().to_string();
    if provider_id.is_empty() {
        return Err("provider_id must not be empty".to_string());
    }

    log::debug!("[providers][list_models] provider_id={}", provider_id);

    // Explicit `cloud_providers` entry wins (e.g. a user-pointed remote
    // ollama box at https://ollama.example.com/v1). Falling back to the
    // local-runtime synthesis below only happens when no entry matches.
    let entry = config
        .cloud_providers
        .iter()
        .find(|e| e.id == provider_id || e.slug == provider_id)
        .cloned()
        .or_else(|| synthesize_local_runtime_entry(&provider_id, config))
        .or_else(|| synthesize_managed_entry(&provider_id))
        .ok_or_else(|| format!("no cloud provider with id or slug '{}' found", provider_id))?;

    // Ordinary saved BYOK keys need the same safety boundary as one-shot
    // candidate keys. Reject unsafe destinations before loading any credential
    // or constructing an authenticated request.
    if matches!(
        entry.auth_style,
        crate::config::schema::cloud_providers::AuthStyle::Bearer
            | crate::config::schema::cloud_providers::AuthStyle::Anthropic
    ) && !crate::inference::provider::factory::credentialed_endpoint_is_safe(&entry.endpoint)
    {
        return Err(
            "Credentialed provider endpoints require HTTPS or loopback HTTP without URL credentials"
                .into(),
        );
    }

    // Candidate credentials are never persisted or mixed with existing account
    // keys. Only explicitly configured BYOK providers can be tested. Reject
    // plaintext remote endpoints and credential-bearing URLs before any I/O.
    if let Some(candidate) = candidate_key {
        let auth_ok = matches!(
            entry.auth_style,
            crate::config::schema::cloud_providers::AuthStyle::Bearer
                | crate::config::schema::cloud_providers::AuthStyle::Anthropic
        );
        let anonymous = candidate.is_empty()
            && entry.auth_style == crate::config::schema::cloud_providers::AuthStyle::None;
        let url = reqwest::Url::parse(&entry.endpoint)
            .map_err(|_| "Candidate probe requires a valid HTTP(S) endpoint".to_string())?;
        if (!auth_ok && !anonymous) || entry.slug == "openhuman" {
            return Err(
                "Candidate credential probing requires an API-key provider or an anonymous endpoint"
                    .into(),
            );
        }
        // Anonymous endpoints may use remote plaintext HTTP because no key is
        // ever sent; credentialed endpoints must use HTTPS or local loopback.
        if !matches!(url.scheme(), "http" | "https")
            || (auth_ok
                && !crate::inference::provider::factory::credentialed_endpoint_is_safe(
                    &entry.endpoint,
                ))
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "Candidate provider requires a valid HTTP(S) URL; credentialed providers require HTTPS or loopback HTTP without URL credentials"
                    .into(),
            );
        }
        if candidate.len() > 256 * 1024 {
            return Err("Candidate credential exceeds 256 KiB".into());
        }
    }

    // Sentry TAURI-RUST-114C/114D: the app stores Claude Code with the cosmetic
    // endpoint `cli://claude-code`. It runs as a local CLI and has no `/models`
    // listing, and reqwest cannot build a request for a non-http(s) URL, so the
    // probe failed with "builder error" on every picker open. Such a provider
    // has no remote catalog: answer with an empty one.
    if !endpoint_hosts_models_listing(&entry.slug, &entry.endpoint) {
        log::debug!(
            "[providers][list_models] slug={} has no http(s) /models listing; returning an empty list",
            entry.slug
        );
        return Ok(crate::core::Outcome::new(
            serde_json::json!({ "models": Vec::<ModelInfo>::new() }),
            vec![format!(
                "provider '{}' has no remote model catalog",
                entry.slug
            )],
        ));
    }

    use crate::config::schema::cloud_providers::AuthStyle;
    let raw_api_key = if let Some(candidate) = candidate_key {
        candidate.to_owned()
    } else {
        let looked_up =
            crate::inference::provider::factory::lookup_key_for_slug(&entry.slug, config)
                .unwrap_or_default();
        resolve_local_runtime_key(&entry.slug, looked_up, config)
    };

    // A candidate API key must never be interpreted as an existing Codex OAuth
    // session or cause the stored OAuth account to enter the probe.
    let bearer_is_oauth = candidate_key.is_none()
        && entry.slug == "openai"
        && crate::inference::provider::factory::openai_bearer_is_oauth(config);
    // Authenticate individual candidates for /models, not a combined bearer.
    // Managed sessions and Codex OAuth are never interpreted as key groups.
    let candidates = if candidate_key.is_some() && raw_api_key.is_empty() {
        // Probe an anonymous/local endpoint without ever loading an existing
        // stored credential: absence of a new key must not leak the old key
        // to a newly edited origin during a preview.
        vec![String::new()]
    } else if candidate_key.is_some() {
        // Keep candidate health separate from committed account health; the
        // probe cannot evict or cool down the actual provider's credentials.
        let keys = crate::inference::provider::factory::model_fleet_probe_keys(
            &format!("{}:{}:candidate-catalog", config.workspace_dir.display(), entry.slug),
            raw_api_key,
        );
        if keys.is_empty() || keys.len() > 128 {
            return Err("Candidate credential group must contain 1-128 distinct keys".into());
        }
        if keys.iter().any(|key| {
            key.len() > 4096 || key.chars().any(char::is_control)
        }) {
            return Err("Candidate credential contains invalid characters or an oversized key".into());
        }
        keys
    } else if matches!(entry.auth_style, AuthStyle::Bearer | AuthStyle::Anthropic)
        && !bearer_is_oauth
    {
        crate::inference::provider::factory::model_fleet_probe_keys(
            &format!("{}:{}:catalog", config.workspace_dir.display(), entry.slug),
            raw_api_key,
        )
    } else {
        vec![raw_api_key]
    };
    let api_key = candidates.first().cloned().unwrap_or_default();

    let routing =
        resolve_openai_codex_routing(config, &entry.slug, &entry.endpoint, &api_key, bearer_is_oauth)
            .unwrap_or_else(|err| {
            log::warn!(
                "[providers][list_models] openai codex routing unavailable; continuing with configured endpoint: {err}"
            );
            OpenAiCodexRouting::standard(&entry.endpoint)
        });

    let mut models_url = format!("{}/models", routing.endpoint);
    let codex_client_version = routing.using_oauth.then(openai_codex_client_version);
    if let Some(client_version) = codex_client_version.as_deref() {
        models_url = append_query_param(&models_url, "client_version", client_version);
    }

    log::debug!(
        "[providers][list_models] fetching url={} slug={} codex_oauth={} account_id_header={}",
        models_url,
        entry.slug,
        routing.using_oauth,
        routing.account_id.is_some()
    );

    // This probe carries provider credentials, including custom x-api-key
    // headers that reqwest does not strip from redirects. Reuse the native
    // no-redirect transport builder with the same TLS roots, proxy and CA.
    let client = crate::util::tls::credentialed_client_builder(
        "providers.list_models",
        config
            .cloud_provider_ca_certs
            .get(entry.slug.trim())
            .map(String::as_str)
            .filter(|pem| !pem.is_empty()),
    )?
    .timeout(std::time::Duration::from_secs(30))
    .connect_timeout(std::time::Duration::from_secs(10))
    .build()
    .map_err(|_| "Could not build secure provider catalogue probe client".to_string())?;

    // Managed backend (`openhuman`) needs a different URL *and* a different
    // credential than every BYOK provider above, so neither `entry.endpoint`
    // nor `lookup_key_for_slug` is usable here:
    //
    //   * the seeded entry's endpoint is the *chat* base
    //     (`https://api.openhuman.ai/v1`), so `{endpoint}/models` would probe
    //     the wrong host entirely — the hosted API is resolved by
    //     `effective_backend_api_url`, which also ignores an `api_url`
    //     override pointing at a local/third-party inference host.
    //   * the session JWT lives in the `app-session` auth profile, not
    //     `provider:openhuman`, so `lookup_key_for_slug` returns "" and the
    //     request would go out unauthenticated (401).
    //
    // `?catalog=openrouter` is required: without it the backend returns only
    // the curated legacy list, which is deliberately
    // byte-identical to the legacy payload. The catalog listing is gated
    // server-side by OPENROUTER_PASSTHROUGH_ENABLED and returns an empty set
    // when the passthrough is off, so this degrades to "no models" rather than
    // an error on a backend that has not enabled it.
    let mut managed_token = String::new();
    if entry.auth_style == AuthStyle::OpenhumanJwt {
        // Do NOT propagate a missing session: a self-hosted entry may carry a
        // provider-scoped key instead, and the auth arm below documents that
        // fallback. Only fail when neither credential exists, so the error the
        // caller sees names the real problem.
        //
        // Classify the session directly rather than going through
        // `require_live_session_token`, which flattens "signed out" and "could
        // not read the credential store" into one opaque Err. A lock timeout or
        // filesystem error is a recoverable fault the picker should surface —
        // swallowing it into a successful empty catalog hides it (review, #6206).
        // A store error still propagates; only a genuinely absent/expired
        // session degrades to an empty list.
        use crate::security::credentials::session_support::{
            classify_session_token, load_app_session_profile, publish_local_session_expiry,
            SessionTokenCheck,
        };
        let profile = load_app_session_profile(config)?;
        match classify_session_token(profile.as_ref(), chrono::Utc::now()) {
            SessionTokenCheck::Live(token) => managed_token = token,
            // Signed out is not a provider failure. The managed catalog has
            // nothing to offer until there is a session, and managed stays
            // selectable on its automatic routing — so return an empty list
            // rather than surfacing "could not load models".
            check if api_key.is_empty() => {
                let reason = match check {
                    SessionTokenCheck::Expired => {
                        // Still announce the expiry: `require_live_session_token`
                        // did this for us before, and without it an expired token
                        // stays in the store with nothing prompting a re-auth.
                        publish_local_session_expiry("list_configured_models");
                        "session expired"
                    }
                    _ => "no session",
                };
                log::info!(
                    "[providers][list_models] managed catalog unavailable — {reason}; returning an empty list"
                );
                return Ok(crate::core::Outcome::new(
                    serde_json::json!({ "models": Vec::<ModelInfo>::new() }),
                    vec![format!("{reason}; managed catalog is empty")],
                ));
            }
            check => {
                if matches!(check, SessionTokenCheck::Expired) {
                    publish_local_session_expiry("list_configured_models");
                }
                log::debug!(
                    "[providers][list_models] no live session; falling back to the provider-scoped key"
                );
            }
        }
        let Ok(base) = crate::backend::base_url(&config.api_url) else {
            log::info!(
                "[providers][list_models] managed catalog unavailable — no backend transport; returning an empty list"
            );
            return Ok(crate::core::Outcome::new(
                serde_json::json!({ "models": Vec::<ModelInfo>::new() }),
                vec!["no hosted backend; managed catalog is empty".to_string()],
            ));
        };
        models_url = append_query_param(
            &crate::util::url::join_url(&base, "/openai/v1/models"),
            "catalog",
            "openrouter",
        );
        log::debug!(
            "[providers][list_models] managed catalog url={}",
            models_url
        );
    }

    // This is a safe read-only GET: when the current key is rejected with
    // 401/403, try only the remaining distinct credentials. A successful
    // catalogue response is proof of at least one accepted key. Do not
    // retry 5xx/timeouts or turn pool cardinality into an auth assertion.
    let mut rejected_keys: usize = 0;
    let mut checked_keys: usize = 0;
    let mut authenticated = false;
    let mut managed_session_attached = false;
    let mut successful_response = None;

    for (position, candidate_key) in candidates.iter().enumerate() {
        let has_more = position + 1 < candidates.len();
        checked_keys += 1;
        // An explicit authenticated /key validation is stronger evidence than
        // a /models response. For every other vendor, compare to an anonymous
        // control request before calling the key verified.
        let using_key_preflight = is_openrouter_provider(&entry.slug, &entry.endpoint);
        if using_key_preflight {
            match validate_openrouter_api_key(&client, &routing.endpoint, candidate_key).await {
                Ok(()) => {}
                Err(error) if should_retry_openrouter_key(&error, has_more) => {
                    rejected_keys += 1;
                    continue;
                }
                Err(error) => return Err(error),
            }
        }

        let mut request = client.get(&models_url);
        if routing.using_oauth {
            request = request
                .header(
                    reqwest::header::USER_AGENT,
                    openai_codex_user_agent("OpenHuman", env!("CARGO_PKG_VERSION")),
                )
                .header(OPENAI_CODEX_ORIGINATOR_HEADER, OPENAI_CODEX_ORIGINATOR);
        }

        request = match entry.auth_style {
            AuthStyle::Bearer => {
                if !candidate_key.is_empty() {
                    let mut r =
                        request.header("Authorization", format!("Bearer {}", candidate_key));
                    if let Some(account_id) = routing.account_id.as_deref() {
                        r = r.header(OPENAI_CODEX_ACCOUNT_HEADER, account_id);
                    }
                    r
                } else {
                    request
                }
            }
            AuthStyle::Anthropic => {
                let mut r = request.header("anthropic-version", "2023-06-01");
                if !candidate_key.is_empty() {
                    r = r.header("x-api-key", candidate_key.as_str());
                }
                r
            }
            AuthStyle::OpenhumanJwt => {
                // Prefer the live session JWT resolved above; fall back to a
                // provider-scoped key so a self-hosted entry that stores one still
                // authenticates.
                let token = if !managed_token.is_empty() {
                    managed_token.as_str()
                } else {
                    candidate_key.as_str()
                };
                managed_session_attached = managed_session_attaches(&managed_token, &models_url);
                // Never put a bearer credential on the wire in clear text. `https`
                // or a loopback host only — loopback stays allowed so a local
                // backend (BACKEND_URL=http://127.0.0.1:...) still works in dev.
                if !token.is_empty() && url_is_credential_safe(&models_url) {
                    // Managed traffic is attributed per embedding product
                    // (OpenCompany / desktop); the generic provider client
                    // does not carry it, so attach it explicitly.
                    request
                        .header("Authorization", format!("Bearer {}", token))
                        .headers(crate::backend::attribution_headers())
                } else if !token.is_empty() {
                    log::warn!(
                        "[providers][list_models] refusing to send a bearer token to a non-https, non-loopback URL"
                    );
                    request
                } else {
                    request
                }
            }
            AuthStyle::None => request,
        };

        let attempt = request
            .send()
            .await
            .map_err(|e| format!("[providers][list_models] HTTP request failed: {}", e))?;

        if should_try_next_catalog_key(attempt.status().as_u16(), has_more) {
            rejected_keys += 1;
            continue;
        }

        authenticated = matches!(entry.auth_style, AuthStyle::Bearer | AuthStyle::Anthropic)
            && !candidate_key.is_empty()
            && attempt.status().is_success()
            && (using_key_preflight
                || catalogue_requires_credential(&client, &models_url, entry.auth_style).await);
        successful_response = Some(attempt);
        break;
    }

    let response = successful_response.ok_or_else(|| {
        "No credential in the provider group produced a catalogue response".to_string()
    })?;

    let status = response.status();
    if !status.is_success() {
        // A 401 from the MANAGED catalog means the caller is not signed in —
        // the stored session was rejected server-side even though its local
        // `exp` was still valid, so `require_live_session_token` handed us a
        // token the backend no longer honours. That is a signed-out state, not
        // a provider failure, and rendering it as "could not load models" put a
        // red error under managed for a user whose only problem is a stale
        // session. Managed stays selectable on its automatic routing, so return
        // an empty catalog and let the app's normal auth surfaces prompt for
        // re-authentication.
        //
        // Scoped to the managed provider on purpose: for a BYOK provider a 401
        // IS the actionable error (a wrong or revoked API key), and hiding it
        // would strand the user with a silently empty dropdown.
        if managed_401_means_signed_out(status.as_u16(), entry.auth_style, managed_session_attached)
        {
            log::info!(
                "[providers][list_models] managed catalog unavailable — backend rejected the session token (401); returning an empty list"
            );
            return Ok(crate::core::Outcome::new(
                serde_json::json!({ "models": Vec::<ModelInfo>::new() }),
                vec!["session not accepted; managed catalog is empty".to_string()],
            ));
        }
        let body = response.text().await.unwrap_or_default();
        let sanitized = sanitize_api_error(&body);
        let truncated = crate::util::truncate_with_ellipsis(&sanitized, 300);
        // TAURI-RUST-8X3: a 404 from the `<base>/models` probe means the
        // configured base URL does not host an OpenAI-compatible `/models`
        // listing (wrong base, a model-only proxy, or a missing `/v1`
        // suffix). Append an actionable hint so the model-dropdown probe
        // surfaces *recovery guidance* inline instead of the bare Go-style
        // `404 page not found`. The `provider returned 404` prefix is kept
        // verbatim so the `is_provider_user_state_message` classifier anchor
        // (which demotes this preventable user-state case out of Sentry)
        // still matches — see `crates/openhuman-core/src/core/observability.rs`.
        if status.as_u16() == 404 {
            return Err(format!(
                "provider returned 404: {} — the configured base URL does not expose a `/models` endpoint; check the provider's base URL (it usually ends in `/v1`)",
                truncated
            ));
        }
        return Err(format!(
            "provider returned {}: {}",
            status.as_u16(),
            truncated
        ));
    }

    // TAURI-RUST-12: `response.json()` discards the body when decoding fails,
    // so Sentry just sees `error decoding response body` with no clue what the
    // server actually sent. In practice the offending body is HTML from a
    // captive portal / corporate proxy login page, an upstream load-balancer
    // 502 served as HTML with a `200 OK`, or a JSON parser tripping on a
    // wrong-path endpoint. Read the body as text first, then parse, and
    // surface a sanitized + truncated snippet so the failure is diagnosable
    // from the error string alone.
    let raw_body = response.text().await.map_err(|e| {
        format!(
            "[providers][list_models] failed to read response body: {}",
            e
        )
    })?;
    let body: serde_json::Value = serde_json::from_str(&raw_body).map_err(|e| {
        let sanitized = sanitize_api_error(&raw_body);
        let snippet = crate::util::truncate_with_ellipsis(&sanitized, 300);
        format!(
            "[providers][list_models] failed to parse JSON: {} (body: {})",
            e, snippet
        )
    })?;

    // OpenAI-compatible servers occasionally return HTTP 200 with an error
    // payload instead of a 4xx (LM Studio does this for unknown paths like
    // `/v11/models` — body `{"error":"Unexpected endpoint or method..."}`).
    // Treat any top-level `error` field as a failure so the AI-panel probe
    // doesn't silently accept a typo'd endpoint.
    if let Some(err_field) = body.get("error") {
        let msg = err_field
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| {
                err_field
                    .get("message")
                    .and_then(|m| m.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_else(|| err_field.to_string());
        let sanitized = sanitize_api_error(&msg);
        return Err(format!("provider returned error payload: {}", sanitized));
    }

    // Parse the OpenAI-compatible `/models` envelope into typed model
    // entries. See `parse_models_response` for the distinct error shapes
    // returned for "missing field" vs "field present but wrong type"
    // (TAURI-RUST-4Y). The ChatGPT Codex backend uses a sibling `models`
    // array keyed by `slug`, so that shape is accepted here too.
    let mut models = parse_models_response(&body).map_err(|error| error.to_string())?;
    if routing.using_oauth {
        merge_openai_codex_model_hints(&mut models);
    }

    log::info!(
        "[providers][list_models] slug={} fetched {} models",
        entry.slug,
        models.len()
    );

    Ok(crate::core::Outcome::new(
        serde_json::json!({
            "models": models,
            "fleet_probe": {
                "authenticated": authenticated,
                "checked_keys": checked_keys,
                "rejected_keys": rejected_keys,
            }
        }),
        vec![format!("fetched {} models", models.len())],
    ))
}

#[cfg(test)]
#[path = "catalog_listing_tests.rs"]
mod model_fleet_probe_tests;
