//! `<slug>:<model>` BYOK cloud providers: shared slug resolution (model fallback,
//! abstract-tier remapping, credentials, codex routing) and the crate-native builders.

use super::{
    access_gates::enforce_local_only_inference, cloud_spec, credentials::CredentialAccess,
    local_runtime::OptionalChatModelResult, managed_backend::managed_chat_model_for,
    openai_bearer_is_oauth, provider_for_role, redact_endpoint_origin, resolve_codex_routing,
    InspectionOutputBudgetCapability, BYOK_INCOMPLETE_SENTINEL, CLAUDE_AGENT_SDK_PREFIX,
    CLAUDE_AGENT_SDK_PROVIDER, PROVIDER_OPENHUMAN,
};
use crate::config::ops::resolve_primary_cloud_provider_string;
use crate::config::schema::cloud_providers::AuthStyle;
use crate::config::Config;
#[cfg(not(test))]
use crate::inference::provider::factory::access_gates::verify_backend_session_active;
#[cfg(not(test))]
use crate::inference::provider::factory::access_gates::verify_session_active;
use crate::inference::provider::fallback_diagnostics;
use crate::inference::provider::openai_codex::resolve_openai_codex_routing_for_inspection;
use crate::security::egress::EgressLedger;
use std::sync::Arc;
use tinyinference_llm::model::ChatModel;
use tinyinference_llm::providers::anthropic::endpoint_is_anthropic_messages;
use tinyinference_llm::providers::openai::codex::{
    openai_codex_client_version, openai_codex_user_agent, OPENAI_CODEX_ACCOUNT_HEADER,
    OPENAI_CODEX_ORIGINATOR, OPENAI_CODEX_ORIGINATOR_HEADER,
};
use tinyinference_llm::providers::registry::resolve_slug_model;
use tinyinference_providers::route::split_model_and_temperature;

/// Use the Codex-resolved endpoint for Bearer routes, including OAuth's
/// chatgpt.com backend. Keeping the saved generic OpenAI endpoint here silently
/// sends Codex-specific headers and tokens to the wrong API base.
pub(super) fn cloud_base_url<'a>(
    entry: &'a crate::config::schema::cloud_providers::CloudProviderCreds,
    codex: &'a tinyinference_llm::providers::openai::codex::OpenAiCodexRouting,
) -> &'a str {
    if entry.auth_style == AuthStyle::Bearer {
        codex.endpoint.as_str()
    } else {
        entry.endpoint.as_str()
    }
}

/// A bearer key for an independently deployed gateway must not travel over
/// remote cleartext HTTP or be mixed into URL userinfo/query/fragment fields.
/// Localhost HTTP is allowed for separately installed development gateways.
pub(super) fn prodex_gateway_endpoint_is_safe(endpoint: &str) -> bool {
    let Ok(url) = reqwest::Url::parse(endpoint) else {
        return false;
    };
    if !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.host_str().is_none()
    {
        return false;
    }
    match url.scheme() {
        "https" => true,
        "http" => {
            let hostname = url.host_str().unwrap_or_default();
            let unbracketed = hostname.trim_start_matches('[').trim_end_matches(']');
            unbracketed.eq_ignore_ascii_case("localhost")
                || unbracketed
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|address| address.is_loopback())
        }
        _ => false,
    }
}

/// Any image route can receive this provider's scoped credential. The
/// config-file loading path must not bypass the settings/RPC HTTPS rule.
/// Validate even currently unused model overrides: future model switches must
/// not silently expose an already-stored key over remote cleartext HTTP.
pub(super) fn image_destinations_are_safe(
    entry: &crate::config::schema::cloud_providers::CloudProviderCreds,
) -> bool {
    entry
        .image_endpoint
        .as_deref()
        .is_none_or(|url| url.trim().is_empty() || prodex_gateway_endpoint_is_safe(url.trim()))
        && entry
            .model_image_endpoints
            .values()
            .all(|url| url.trim().is_empty() || prodex_gateway_endpoint_is_safe(url.trim()))
}

/// Prodex also requires a protected main gateway origin because its virtual
/// key is not a general-purpose direct provider credential.
pub(super) fn prodex_gateway_destinations_are_safe(
    entry: &crate::config::schema::cloud_providers::CloudProviderCreds,
) -> bool {
    prodex_gateway_endpoint_is_safe(&entry.endpoint) && image_destinations_are_safe(entry)
}

/// The downstream gateway owns provider/account selection and sticky retry
/// affinity. A second OpenHuman key fleet for this slug would fragment quota
/// accounting and might send a provider continuation to another gateway key.
pub(super) fn prodex_has_single_virtual_key(raw: &str) -> bool {
    let keys = super::key_pool::parse_api_keys(raw);
    keys.len() == 1 && !keys[0].trim().is_empty()
}

type CloudSlugInspectionChatModelResult = Option<
    anyhow::Result<(
        Arc<dyn ChatModel<()>>,
        String,
        String,
        String,
        InspectionOutputBudgetCapability,
    )>,
>;

#[path = "cloud_slug_resolution.rs"]
mod resolution;
#[cfg(test)]
pub(super) use resolution::lookup_failure;
#[cfg(test)]
pub(super) use resolution::resolve_cloud_slug;
use resolution::{inspection_output_budget_capability, resolve_cloud_slug_with_access};
pub(crate) use resolution::{resolve_cloud_slug_for, CloudSlugResolution};

/// A `<slug>:<model>` BYOK cloud provider as a crate-native [`ChatModel`] — the
/// Native model for every configured cloud auth style, including the managed
/// `OpenhumanJwt` entry (issue #4727 Phase 3).
///
/// Returns `None` unless the role resolves to a **configured** cloud slug. When
/// it does:
/// - `Anthropic` / `None` / plain `Bearer` → crate `OpenAiModel` Chat Completions;
/// - `Bearer` with OpenAI **Codex OAuth** → crate `OpenAiModel` on the Responses
///   API (`with_responses_api_primary`), with the codex account/originator
///   headers, user-agent, `client_version` query param, and `max_output_tokens`
///   omitted (the crate `/v1/responses` support, tinyagents#51);
/// - `OpenhumanJwt` → the crate-native managed backend model.
///
/// The legacy host's rare chat-completions-404 → `/v1/responses` **fallback** for
/// non-codex slugs is not replicated (the crate has responses-*primary*, not
/// fallback); chat completions is the primary path those slugs use in practice.
///
/// The resolution is shared via [`resolve_cloud_slug_for`], so slugs resolve
/// identically to the legacy path; only the wire client differs. The **same**
/// access gate the `Provider` path applies (`enforce_local_only_inference` +
/// `verify_session_active`) runs before building. Temperature rides the per-call
/// `ModelRequest` (managed/local parity; the `@<temp>` suffix still bakes a fixed
/// override).
#[cfg(test)]
pub(super) fn try_create_cloud_slug_chat_model(
    role: &str,
    config: &Config,
    egress: &EgressLedger,
) -> OptionalChatModelResult {
    let extensions = crate::core::runtime::CoreContext::current_extensions();
    try_create_cloud_slug_chat_model_with_native_tools_for(
        role,
        config,
        true,
        egress,
        extensions.as_ref(),
    )
}

pub(super) fn try_create_cloud_slug_chat_model_with_native_tools_for(
    role: &str,
    config: &Config,
    native_tool_calling: bool,
    egress: &EgressLedger,
    extensions: Option<&crate::core::slots::Extensions>,
) -> OptionalChatModelResult {
    try_create_cloud_slug_chat_model_with_native_tools_and_access(
        role,
        config,
        native_tool_calling,
        egress,
        extensions,
        CredentialAccess::Runtime,
        None,
    )
}

pub(super) fn try_create_cloud_slug_chat_model_for_inspection_with_identity(
    role: &str,
    config: &Config,
    extensions: Option<&crate::core::slots::Extensions>,
) -> CloudSlugInspectionChatModelResult {
    let mut identity = None;
    let mut resolved = provider_for_role(role, config);
    if resolved.trim().is_empty() || resolved.trim() == "cloud" {
        resolved = resolve_primary_cloud_provider_string(config);
    }
    let result = try_create_cloud_slug_chat_model_from_string_with_native_tools_and_access(
        role,
        &resolved,
        config,
        true,
        &EgressLedger::none(),
        extensions,
        CredentialAccess::Inspection,
        Some(&mut identity),
    )?;
    Some(result.and_then(|(model, model_id)| {
        let (endpoint, credential, wire_model, output_budget) = identity.ok_or_else(|| {
            anyhow::anyhow!("inspection route did not expose its frozen wire identity")
        })?;
        if wire_model != model_id {
            anyhow::bail!("inspection route model identity did not match its constructed model");
        }
        Ok((model, model_id, endpoint, credential, output_budget))
    }))
}

fn try_create_cloud_slug_chat_model_with_native_tools_and_access(
    role: &str,
    config: &Config,
    native_tool_calling: bool,
    egress: &EgressLedger,
    extensions: Option<&crate::core::slots::Extensions>,
    credential_access: CredentialAccess,
    inspection_identity: Option<
        &mut Option<(String, String, String, InspectionOutputBudgetCapability)>,
    >,
) -> OptionalChatModelResult {
    // Resolve the role's provider string, expanding the empty / "cloud" sentinel
    // to the primary cloud target.
    let mut resolved = provider_for_role(role, config);
    if resolved.trim().is_empty() || resolved.trim() == "cloud" {
        resolved = resolve_primary_cloud_provider_string(config);
    }
    try_create_cloud_slug_chat_model_from_string_with_native_tools_and_access(
        role,
        &resolved,
        config,
        native_tool_calling,
        egress,
        extensions,
        credential_access,
        inspection_identity,
    )
}

pub(super) fn try_create_cloud_slug_chat_model_from_string(
    role: &str,
    provider: &str,
    config: &Config,
    egress: &EgressLedger,
) -> OptionalChatModelResult {
    try_create_cloud_slug_chat_model_from_string_with_native_tools(
        role, provider, config, true, egress,
    )
}

pub(super) fn try_create_cloud_slug_chat_model_from_string_with_native_tools(
    role: &str,
    provider: &str,
    config: &Config,
    native_tool_calling: bool,
    egress: &EgressLedger,
) -> OptionalChatModelResult {
    let extensions = crate::core::runtime::CoreContext::current_extensions();
    try_create_cloud_slug_chat_model_from_string_with_native_tools_and_access(
        role,
        provider,
        config,
        native_tool_calling,
        egress,
        extensions.as_ref(),
        CredentialAccess::Runtime,
        None,
    )
}

pub(super) fn try_create_cloud_slug_chat_model_from_string_with_native_tools_for(
    role: &str,
    provider: &str,
    config: &Config,
    native_tool_calling: bool,
    egress: &EgressLedger,
    extensions: Option<&crate::core::slots::Extensions>,
) -> OptionalChatModelResult {
    try_create_cloud_slug_chat_model_from_string_with_native_tools_and_access(
        role,
        provider,
        config,
        native_tool_calling,
        egress,
        extensions,
        CredentialAccess::Runtime,
        None,
    )
}

fn try_create_cloud_slug_chat_model_from_string_with_native_tools_and_access(
    role: &str,
    provider: &str,
    config: &Config,
    native_tool_calling: bool,
    egress: &EgressLedger,
    extensions: Option<&crate::core::slots::Extensions>,
    credential_access: CredentialAccess,
    mut inspection_identity: Option<
        &mut Option<(String, String, String, InspectionOutputBudgetCapability)>,
    >,
) -> OptionalChatModelResult {
    let p = provider.trim().to_string();

    // Only the "<slug>:<model>[@temp]" cloud form routes here. The managed
    // backend, BYOK-incomplete sentinel, and bespoke subprocess providers
    // (claude-code / claude_agent_sdk) are handled on the `Provider` path.
    if p == PROVIDER_OPENHUMAN
        || p == BYOK_INCOMPLETE_SENTINEL
        || p == CLAUDE_AGENT_SDK_PROVIDER
        || p.starts_with(CLAUDE_AGENT_SDK_PREFIX)
        || p.starts_with(tinyagents_harness::providers::claude_code::PROVIDER_PREFIX)
    {
        return None;
    }
    let colon = p.find(':')?;
    let slug = p[..colon].trim().to_string();
    if slug.is_empty() {
        return None;
    }
    let (raw_model, temperature_override) = split_model_and_temperature(&p[colon + 1..]);
    // Not a configured cloud slug → let the `Provider` path surface the precise
    // "no cloud provider configured" error rather than silently no-op'ing.
    if !config.cloud_providers.iter().any(|e| e.slug == slug) {
        return None;
    }

    // Preserve the `Provider` path's gate for custom/cloud providers.
    if let Err(e) = enforce_local_only_inference(role, &p) {
        return Some(Err(e));
    }
    if credential_access == CredentialAccess::Runtime {
        #[cfg(not(test))]
        if let Err(e) = verify_session_active(config) {
            return Some(Err(e));
        }
    }

    let CloudSlugResolution {
        entry,
        effective_model,
        key,
        codex,
    } = match resolve_cloud_slug_with_access(
        role,
        &slug,
        &raw_model,
        config,
        extensions,
        credential_access,
    ) {
        Ok(r) => r,
        Err(e) => return Some(Err(e)),
    };

    // A Codex subscription bearer is scoped to the official Responses API.
    // Never forward that bearer to user-configured image endpoints, even if
    // those settings were saved while an API-key profile was active.
    if codex.using_oauth
        && (entry
            .image_endpoint
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty())
            || entry
                .model_image_endpoints
                .values()
                .any(|value| !value.trim().is_empty()))
    {
        return Some(Err(anyhow::anyhow!(
            "Codex OAuth credentials cannot be sent to alternate image endpoints"
        )));
    }
    let codex_accounts = if codex.using_oauth && credential_access == CredentialAccess::Runtime {
        match crate::security::credentials::openai_oauth::lookup_openai_oauth_fleet(config) {
            Ok(accounts) => accounts,
            Err(error) => return Some(Err(anyhow::anyhow!(error))),
        }
    } else {
        Vec::new()
    };

    // Credentials are parsed once; one provider slug can own a pool of keys.
    // No credential group is ever sent as a single upstream Authorization header.
    let keys = if credential_access == CredentialAccess::Inspection {
        vec![key.clone()]
    } else if codex.using_oauth {
        if codex_accounts.is_empty() {
            vec![key.clone()]
        } else {
            codex_accounts
                .iter()
                .map(|account| account.access_token.clone())
                .collect()
        }
    } else if matches!(entry.auth_style, AuthStyle::Bearer | AuthStyle::Anthropic) {
        match super::credentials::lookup_token_account_fleet(&slug, config, &key, false) {
            Ok(found) if !found.is_empty() => found,
            Ok(_) => vec![String::new()],
            Err(error) => return Some(Err(error)),
        }
    } else {
        vec![key.clone()]
    };
    if slug == "prodex" && !prodex_has_single_virtual_key(&key) {
        return Some(Err(anyhow::anyhow!(
            "Prodex Gateway requires exactly one inference-scoped virtual key; rotate upstream provider accounts in Prodex"
        )));
    }
    if codex.using_oauth && keys.len() > 1 && keys.len() != codex_accounts.len() {
        return Some(Err(anyhow::anyhow!(
            "OpenAI Codex OAuth credentials cannot be mixed with API keys"
        )));
    }
    let image_endpoint = entry
        .model_image_endpoints
        .get(&effective_model)
        .or(entry.image_endpoint.as_ref())
        .map(String::as_str)
        .map(str::trim)
        .filter(|url| {
            !url.is_empty() && url.trim_end_matches('/') != entry.endpoint.trim_end_matches('/')
        });

    let fleet_scope = format!(
        "{}:{}:{}",
        config.workspace_dir.display(),
        slug,
        effective_model
    );
    let strategy = super::key_pool::RotationStrategy::parse(
        entry.rotation_strategy.as_deref().unwrap_or("round_robin"),
    );

    // Every configured cloud slug builds a crate-native model through the
    // registry (`cloud_spec`). OpenhumanJwt delegates to the managed backend
    // model; Codex OAuth routes to the Responses API with its headers / UA /
    // query; every other Bearer/Anthropic/None slug uses Chat Completions (or
    // the Anthropic Messages adapter for a first-party native-tools endpoint).
    let mut extra_headers: Vec<(String, String)> = config
        .ephemeral_route
        .as_ref()
        .filter(|route| {
            slug == crate::config::schema::ephemeral_route::EPHEMERAL_ROUTE_SLUG
                && route.endpoint == entry.endpoint
        })
        .map(|route| route.headers.clone())
        .unwrap_or_default();
    let default_provider_options = config
        .ephemeral_route
        .as_ref()
        .filter(|route| {
            slug == crate::config::schema::ephemeral_route::EPHEMERAL_ROUTE_SLUG
                && route.endpoint == entry.endpoint
        })
        .and_then(|route| route.provider_options.clone());
    let mut extra_query_params: Vec<(String, String)> = Vec::new();
    let mut user_agent: Option<String> = None;
    let mut codex_route: Option<&tinyinference_llm::providers::openai::codex::OpenAiCodexRouting> =
        None;

    let anthropic_native = entry.auth_style == AuthStyle::Anthropic
        && native_tool_calling
        && endpoint_is_anthropic_messages(&entry.endpoint);
    match entry.auth_style {
        AuthStyle::Anthropic if !anthropic_native => {
            // Anthropic's OpenAI-compatibility endpoint does not support prompt
            // caching, so native tool calling routes to the Messages adapter
            // (which places `cache_control` breakpoints). Text mode
            // (prompt-guided tools) is only implemented on the Chat
            // Completions adapter, so that rare case keeps the compat path.
            log::debug!(
                "[providers][chat-factory] slug={slug} auth_style=anthropic native_tools=false → OpenAI-compatible text-mode path (no prompt caching)"
            );
        }
        AuthStyle::Anthropic | AuthStyle::None => {}
        style
            if crate::config::schema::cloud_providers::is_hosted_auth_style_for(
                extensions, style,
            ) =>
        {
            if credential_access == CredentialAccess::Runtime {
                #[cfg(not(test))]
                if let Err(error) = verify_backend_session_active(config) {
                    return Some(Err(error));
                }
            }
            let model_override =
                (!effective_model.trim().is_empty()).then_some(effective_model.as_str());
            return Some(match credential_access {
                CredentialAccess::Runtime => managed_chat_model_for(
                    extensions,
                    role,
                    config,
                    model_override,
                    None,
                    native_tool_calling,
                    None,
                    egress,
                ),
                CredentialAccess::Inspection => {
                    match super::managed_backend::managed_chat_model_build_for_inspection(
                        extensions,
                        role,
                        config,
                        model_override,
                    ) {
                        Ok(build) => {
                            if let Some(identity) = inspection_identity.as_deref_mut() {
                                *identity = Some((
                                    build.endpoint.clone(),
                                    build.credential.clone(),
                                    build.model_id.clone(),
                                    InspectionOutputBudgetCapability::Bounded,
                                ));
                            }
                            Ok((build.model, build.model_id))
                        }
                        Err(error) => Err(error),
                    }
                }
            });
        }
        AuthStyle::Bearer => {
            // The codex routing may re-target the endpoint (OAuth backend).
            if let Some(account_id) = codex.account_id.as_deref() {
                extra_headers.push((
                    OPENAI_CODEX_ACCOUNT_HEADER.to_string(),
                    account_id.to_string(),
                ));
            }
            if codex.using_oauth {
                // Codex OAuth → Responses API primary + the codex request shape.
                extra_headers.push((
                    OPENAI_CODEX_ORIGINATOR_HEADER.to_string(),
                    OPENAI_CODEX_ORIGINATOR.to_string(),
                ));
                user_agent = Some(openai_codex_user_agent(
                    "OpenHuman",
                    env!("CARGO_PKG_VERSION"),
                ));
                extra_query_params
                    .push(("client_version".to_string(), openai_codex_client_version()));
            }
            codex_route = Some(&codex);
        }
        AuthStyle::OpenhumanJwt => {
            return Some(Err(anyhow::anyhow!(
                "hosted cloud authentication is unavailable without a registered host provider"
            )));
        }
    }

    // Egress spine (privacy epic S2, #4436): committed to a BYOK cloud slug here
    // — past the managed/bespoke returns and the access
    // gates, so this constructs. Disclose as external. Single cloud chokepoint
    // for every cloud ChatModel/turn entry.
    crate::security::egress::emit_external_transfer_in(
        egress,
        crate::security::egress::EgressDescriptor::inference(&slug, &effective_model, true),
    );

    // Always pin credentialed native inference requests to their configured
    // origin. Redirects could carry custom authentication headers (x-api-key)
    // to another URL, because reqwest only strips a fixed header allowlist.
    // This preserves platform TLS, user proxy routing and optional private CAs.
    let service_key = if anthropic_native {
        "provider.anthropic"
    } else {
        "provider.compatible"
    };
    let http = match crate::util::tls::credentialed_client_builder(
        service_key,
        config
            .cloud_provider_ca_certs
            .get(&slug)
            .map(String::as_str)
            .filter(|pem| !pem.is_empty()),
    )
    .and_then(|builder| {
        builder
            .build()
            .map_err(|_| "Could not build secure provider inference HTTP client".to_string())
    }) {
        Ok(client) => Some(client),
        Err(error) => return Some(Err(anyhow::anyhow!(error))),
    };
    let auth_style = cloud_spec::registry_auth_style(entry.auth_style)
        .expect("managed JWT entries returned above");

    // Keep the upstream native ProviderRegistry adapter for every credential.
    // CredentialFleetModel independently selects a healthy key for each call;
    // it never forwards a delimited credential group to any single provider.
    let build = |credential: &str, base: &str| -> anyhow::Result<Arc<dyn ChatModel<()>>> {
        let mut spec = cloud_spec::cloud_provider_spec(entry, &effective_model);
        spec.base_url = base.to_string();
        // Each OAuth model has the account-id header that belongs to its own
        // access token. Reusing the active account's header for every key would
        // cause cross-account requests and broken failover.
        let account = if codex.using_oauth {
            codex_accounts.iter().find(|a| a.access_token == credential)
        } else {
            None
        };
        let mut headers = extra_headers.clone();
        let account_route = account.map(|identity| {
            tinyinference_llm::providers::openai::codex::OpenAiCodexRouting {
                endpoint: codex.endpoint.clone(),
                using_oauth: true,
                account_id: identity.account_id.clone(),
            }
        });
        if codex.using_oauth {
            headers.retain(|(name, _)| !name.eq_ignore_ascii_case(OPENAI_CODEX_ACCOUNT_HEADER));
            if let Some(id) = account.and_then(|identity| identity.account_id.as_deref()) {
                headers.push((OPENAI_CODEX_ACCOUNT_HEADER.to_owned(), id.to_owned()));
            }
        }
        let native = cloud_spec::build_cloud_model(cloud_spec::CloudBuild {
            spec,
            transport: entry.model_transport(&effective_model),
            auth_style: auth_style.clone(),
            key: credential,
            temperature_override,
            temperature_unsupported_models: config.temperature_unsupported_models.as_slice(),
            native_tool_calling,
            default_provider_options: default_provider_options.clone(),
            extra_headers: headers.as_slice(),
            extra_query_params: extra_query_params.as_slice(),
            user_agent: user_agent.as_deref(),
            codex: account_route.as_ref().or(codex_route),
            http: http.clone(),
        })?;
        if entry.tool_call_codec.is_native() {
            Ok(native)
        } else {
            Ok(Arc::new(super::tool_markup::TextToolCompatModel::new(
                native,
                entry.tool_call_codec,
            )))
        }
    };
    let models: Vec<Arc<dyn ChatModel<()>>> = match keys
        .iter()
        .map(|credential| build(credential, cloud_base_url(entry, &codex)))
        .collect()
    {
        Ok(models) => models,
        Err(error) => return Some(Err(error)),
    };
    let image_models: Option<Vec<Arc<dyn ChatModel<()>>>> = match image_endpoint {
        Some(endpoint) => match keys
            .iter()
            .map(|credential| build(credential, endpoint))
            .collect()
        {
            Ok(models) => Some(models),
            Err(error) => return Some(Err(error)),
        },
        None => None,
    };
    let chat = wrap_fleet(fleet_scope, keys, models, image_models, strategy);
    if let Some(identity) = inspection_identity {
        let credential = if entry.auth_style == AuthStyle::None {
            String::new()
        } else {
            key.clone()
        };
        let output_budget = inspection_output_budget_capability(&codex);
        // Doctor's live probe deduplicates on this identity, so it carries
        // everything else that shapes the request: two aliases with the same
        // endpoint, key and model but another auth style, header or CA bundle
        // are two different requests.
        let transport = format!(
            "{:?}\u{1f}{extra_headers:?}\u{1f}{extra_query_params:?}\u{1f}{user_agent:?}\u{1f}{:?}",
            entry.auth_style,
            config.cloud_provider_ca_certs.get(&slug),
        );
        *identity = Some((
            format!("{}\u{1f}{transport}", codex.endpoint),
            credential,
            effective_model.clone(),
            output_budget,
        ));
    }
    Some(Ok((chat, effective_model)))
}

fn wrap_fleet(
    scope: String,
    keys: Vec<String>,
    mut models: Vec<Arc<dyn ChatModel<()>>>,
    image_models: Option<Vec<Arc<dyn ChatModel<()>>>>,
    strategy: super::key_pool::RotationStrategy,
) -> Arc<dyn ChatModel<()>> {
    if models.len() == 1 && image_models.is_none() {
        return models.remove(0);
    }
    Arc::new(super::credential_fleet::CredentialFleetModel::new(
        scope,
        keys,
        models,
        image_models,
        strategy,
    ))
}
