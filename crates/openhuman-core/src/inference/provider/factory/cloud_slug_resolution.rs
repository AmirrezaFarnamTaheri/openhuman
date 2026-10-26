use super::*;

/// Look up a `cloud_providers` entry by slug and build the provider.
/// The shared resolution for a `<slug>:<model>` cloud provider — the cloud
/// `cloud_providers` entry, the effective model id (with `default_model`
/// fallback + abstract-tier remapping), the resolved API key, and the OpenAI
/// codex-oauth routing shared by every cloud `ChatModel` constructor.
pub(crate) struct CloudSlugResolution<'a> {
    pub(crate) entry: &'a crate::config::schema::cloud_providers::CloudProviderCreds,
    pub(crate) effective_model: String,
    pub(crate) key: String,
    pub(crate) codex: tinyinference_llm::providers::openai::codex::OpenAiCodexRouting,
}

pub(crate) fn inspection_output_budget_capability(
    codex: &tinyinference_llm::providers::openai::codex::OpenAiCodexRouting,
) -> InspectionOutputBudgetCapability {
    if codex.using_oauth {
        InspectionOutputBudgetCapability::Unsupported
    } else {
        InspectionOutputBudgetCapability::Bounded
    }
}

pub(crate) fn resolve_cloud_slug_for<'a>(
    role: &str,
    slug: &str,
    model: &str,
    config: &'a Config,
    extensions: Option<&crate::core::slots::Extensions>,
) -> anyhow::Result<CloudSlugResolution<'a>> {
    resolve_cloud_slug_with_access(
        role,
        slug,
        model,
        config,
        extensions,
        CredentialAccess::Runtime,
    )
}

#[cfg(test)]
pub(crate) fn resolve_cloud_slug<'a>(
    role: &str,
    slug: &str,
    model: &str,
    config: &'a Config,
) -> anyhow::Result<CloudSlugResolution<'a>> {
    let extensions = crate::core::runtime::CoreContext::current_extensions();
    resolve_cloud_slug_for(role, slug, model, config, extensions.as_ref())
}

/// The user-facing error for a failed credential lookup: the guidance, plus
/// the expiry signal when the lookup reported one. The doctor's
/// refresh-required check and the Codex reconnect classifier key on those
/// fixed phrases; the rest of the lookup error can carry stored-credential
/// content or a refresh response's body, so it is dropped.
pub(crate) fn lookup_failure(error: &anyhow::Error, guidance: String) -> anyhow::Error {
    let text = format!("{error:#}");
    let signal = if text.contains(super::super::credentials::OAUTH_REFRESH_REQUIRED) {
        Some(super::super::credentials::OAUTH_REFRESH_REQUIRED)
    } else if tinyinference_llm::failure::is_codex_token_expired_text(&text.to_ascii_lowercase()) {
        Some(CODEX_TOKEN_EXPIRED)
    } else {
        None
    };
    match signal {
        Some(signal) => anyhow::anyhow!("{guidance} (underlying error: {signal})"),
        None => anyhow::anyhow!("{guidance}"),
    }
}

/// The Codex OAuth expiry phrase (`openai_oauth::store`) the reconnect
/// classifier recognises.
const CODEX_TOKEN_EXPIRED: &str = "Codex authentication token is expired";

pub(crate) fn resolve_cloud_slug_with_access<'a>(
    role: &str,
    slug: &str,
    model: &str,
    config: &'a Config,
    extensions: Option<&crate::core::slots::Extensions>,
    credential_access: CredentialAccess,
) -> anyhow::Result<CloudSlugResolution<'a>> {
    let entry = config.cloud_providers.iter().find(|e| e.slug == slug);

    let entry = entry.ok_or_else(|| {
        let known: Vec<&str> = config
            .cloud_providers
            .iter()
            .map(|e| e.slug.as_str())
            .collect();
        anyhow::anyhow!(
            "[chat-factory] no cloud provider configured for slug '{}' (role '{}') — \
             add an entry with that slug to cloud_providers in config.toml. \
             Configured slugs: [{}]",
            slug,
            role,
            known.join(", ")
        )
    })?;

    // The gateway virtual key is not a general cloud API token. Never send it
    // to remote HTTP or allow the endpoint to carry URL-embedded credentials.
    if slug == "prodex" && !prodex_gateway_destinations_are_safe(entry) {
        anyhow::bail!(
            "[chat-factory] Prodex gateway endpoints require HTTPS or loopback HTTP, with no URL credentials, query or fragment"
        );
    }
    if !image_destinations_are_safe(entry) {
        anyhow::bail!(
            "[chat-factory] image endpoint overrides require HTTPS or loopback HTTP, with no URL credentials, query or fragment"
        );
    }
    // This is not Prodex-specific: every Bearer/Anthropic direct-provider
    // route can forward a stored account secret. Validate the main endpoint
    // even for config.toml imports and dormant model profiles.
    if matches!(entry.auth_style, AuthStyle::Bearer | AuthStyle::Anthropic)
        && !prodex_gateway_endpoint_is_safe(&entry.endpoint)
    {
        anyhow::bail!(
            "[chat-factory] credentialed provider endpoints require HTTPS or loopback HTTP, with no URL credentials, query or fragment"
        );
    }

    let hosted = crate::config::schema::cloud_providers::is_hosted_auth_style_for(
        extensions,
        entry.auth_style,
    );
    // Hosted routes derive their model from the managed provider and retain the
    // old permissive behavior. BYOK model fallback and hint classification are
    // shared with the provider owner.
    let effective_model = if hosted {
        if model.trim().is_empty() {
            entry.default_model.clone().unwrap_or_default()
        } else {
            model.to_string()
        }
    } else {
        match resolve_slug_model(model, entry.default_model.as_deref(), |candidate| {
            crate::inference::provider_model::is_abstract_model_for(
                extensions,
                PROVIDER_OPENHUMAN,
                candidate,
            )
        }) {
            Ok(model) => model,
            Err(tinyinference_llm::providers::registry::SlugModelError::MissingModel) => {
                log::warn!(
                    "[providers][chat-factory] role={} slug={} resolved to empty model — \
                     provider string must include a model id (e.g. '{}:<model-id>') or \
                     set default_model on the cloud_providers entry",
                    role,
                    slug,
                    slug,
                );
                anyhow::bail!(
                    "[chat-factory] no model configured: role '{}' resolved to an empty model id for slug '{}'. \
                     Include a model in the provider string (e.g. '{slug}:<model-id>') or \
                     set default_model on the cloud_providers entry for slug '{slug}'.",
                    role,
                    slug,
                );
            }
            Err(tinyinference_llm::providers::registry::SlugModelError::NoConcreteDefault) => {
                let requested_model = if model.trim().is_empty() {
                    entry.default_model.as_deref().unwrap_or_default()
                } else {
                    model
                };
                anyhow::bail!(
                    "[chat-factory] model '{}' is an abstract tier for role '{}', \
                     but cloud provider slug '{}' has no concrete default_model configured. \
                     Set cloud_providers[].default_model to a provider-native model id (e.g. deepseek-v4-pro).",
                    requested_model,
                    role,
                    slug
                );
            }
        }
    };

    if !hosted && effective_model != model && !model.trim().is_empty() {
        log::info!(
            "[providers][chat-factory] role={} slug={} remapping abstract model {} -> {}",
            role,
            slug,
            model,
            effective_model
        );
    }

    log::info!(
        "[providers][chat-factory] role={} slug={} model={} endpoint_host={}",
        role,
        slug,
        effective_model,
        redact_endpoint_origin(&entry.endpoint)
    );

    // #5146 §2.1: a raw "failed to read API key for slug 'anthropic'" is
    // baffling when the user never configured Anthropic — they set a local
    // Ollama model and this is a background role that fell back to the cloud.
    // Attach the role, and the local chat model that caused the fallback, so
    // the message explains itself and names a concrete remedy.
    // Only an *implicit* fallback is explained as one. A role with its own
    // explicit cloud route can fail key lookup here too, and telling that user
    // their local chat model caused it would be a lie.
    let fallback_diagnostic = super::super::fallback_diagnostics_for_role(role, config);
    let implicit_fallback = fallback_diagnostic.is_some();
    let local_chat = if implicit_fallback {
        config
            .chat_provider
            .as_deref()
            .filter(|chat| tinyinference_local::profile::is_local_provider_string(chat))
    } else {
        None
    };
    let missing_credentials = || {
        // Safe fields only: role, slug, and the routing shape. Never the
        // underlying error (it can echo a key) and never the key itself.
        log::warn!(
            "[providers][chat-factory] credential lookup failed role={} slug={} auth_style={} implicit_cloud_fallback={}",
            role,
            slug,
            entry.auth_style.as_str(),
            implicit_fallback
        );
        fallback_diagnostic.as_ref().map_or_else(
            || format!("No usable credentials for '{slug}'. Add a key in Connections → LLM."),
            |diagnostic| {
                fallback_diagnostics::missing_provider_credentials_message(
                    diagnostic, slug, local_chat,
                )
            },
        )
    };

    let (key, bearer_is_oauth, oauth_account_id) = if credential_access
        == CredentialAccess::Inspection
        && entry.auth_style == AuthStyle::OpenhumanJwt
    {
        // The managed model constructor receives its credential later, on
        // invoke. Its hosted construction callback is config-only and offline.
        (String::new(), false, None)
    } else if credential_access == CredentialAccess::Inspection {
        let credential = super::super::credentials::lookup_key_for_slug_read_only(slug, config)
            .map_err(|error| lookup_failure(&error, missing_credentials()))?;
        (credential.key, credential.is_oauth, credential.account_id)
    } else {
        let key = credential_access
            .lookup_key_for_slug(slug, config)
            .map_err(|error| lookup_failure(&error, missing_credentials()))?;
        let is_oauth = slug == "openai" && credential_access.openai_bearer_is_oauth(config);
        (key, is_oauth, None)
    };

    // A readable auth profile with no key for this slug returns `Ok("")`, which
    // would otherwise build a client with an empty bearer and surface as a raw
    // 401 from the provider several layers later — exactly the baffling error
    // this diagnostic exists to replace.
    //
    // Runtime construction only rejects missing credentials on the implicit
    // fallback path: explicit routes may be constructed before a key is saved.
    // Inspection is stricter because doctor must report an unusable authenticated
    // route without invoking the model.
    //
    // Styles that carry no stored key (`OpenhumanJwt` injects a session JWT
    // downstream, `None` sends no auth header at all) are legitimately blank and
    // never trip this.
    if (implicit_fallback || credential_access == CredentialAccess::Inspection)
        && key.trim().is_empty()
        && matches!(entry.auth_style, AuthStyle::Bearer | AuthStyle::Anthropic)
    {
        anyhow::bail!("{}", missing_credentials());
    }
    let codex = match credential_access {
        CredentialAccess::Runtime => {
            let bearer_is_oauth = slug == "openai" && openai_bearer_is_oauth(config);
            resolve_codex_routing(config, slug, &entry.endpoint, &key, bearer_is_oauth)
                .map_err(anyhow::Error::msg)?
        }
        CredentialAccess::Inspection => resolve_openai_codex_routing_for_inspection(
            slug,
            &entry.endpoint,
            bearer_is_oauth,
            oauth_account_id,
        ),
    };

    // Re-check the actual endpoint after Codex routing, not just the saved
    // base, so an alternate transport cannot downgrade the credential.
    if matches!(entry.auth_style, AuthStyle::Bearer | AuthStyle::Anthropic)
        && !prodex_gateway_endpoint_is_safe(cloud_base_url(entry, &codex))
    {
        anyhow::bail!(
            "[chat-factory] resolved credential endpoint requires HTTPS or loopback HTTP"
        );
    }

    Ok(CloudSlugResolution {
        entry,
        effective_model,
        key,
        codex,
    })
}
