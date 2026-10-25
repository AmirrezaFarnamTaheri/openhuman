//! Cloud provider credential schema.
//!
//! Each entry in `Config::cloud_providers` represents one configured LLM
//! backend. Providers are keyed by a user-chosen `slug` (e.g. `"openai"`,
//! `"my-deepseek"`). Inference routing in `config::ops::primary_cloud`
//! resolves workload-to-provider strings against this list at runtime using
//! the grammar `"<slug>:<model>"`.
//!
//! Legacy provider discriminators are decoded by the schema-14 config migration.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use std::sync::Arc;

use crate::core::slots::HookSlot;
use tinyinference_providers::builtin::{
    BuiltinAuthStyle, BuiltinCloudProvider as UpstreamBuiltinCloudProvider, BuiltinPresets,
    BUILTIN_CLOUD_PROVIDERS as UPSTREAM_BUILTINS,
};

static HOSTED_PRESETS: HookSlot<BuiltinCloudProvider> =
    HookSlot::new("config::hosted_cloud_presets");
pub type HostedCloudIdentityClassifier = Arc<dyn Fn(&str) -> bool + Send + Sync>;
pub type HostedAuthStyleClassifier = Arc<dyn Fn(AuthStyle) -> bool + Send + Sync>;
static HOSTED_CLOUD_IDENTITY: HookSlot<HostedCloudIdentityClassifier> =
    HookSlot::new("config::hosted_cloud_identity");
static HOSTED_AUTH_STYLE: HookSlot<HostedAuthStyleClassifier> =
    HookSlot::new("config::hosted_auth_style");

/// A built-in cloud provider preset in the host's [`AuthStyle`] vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinCloudProvider {
    pub slug: &'static str,
    pub label: &'static str,
    pub endpoint: &'static str,
    pub auth_style: AuthStyle,
}

/// Register an additional cloud preset supplied by a host.
pub fn register_hosted_cloud_preset(id: impl Into<String>, preset: BuiltinCloudProvider) {
    HOSTED_PRESETS.register(id, preset);
}

/// Register a host preset for one runtime context.
pub fn register_hosted_cloud_preset_in(
    extensions: &crate::core::slots::Extensions,
    id: impl Into<String>,
    preset: BuiltinCloudProvider,
) {
    HOSTED_PRESETS.register_in(extensions, id, preset);
}

/// Register host-owned auth-style classification without making it core
/// inference policy.
pub fn register_hosted_cloud_provider_policy(
    identity: HostedCloudIdentityClassifier,
    auth_style: HostedAuthStyleClassifier,
) {
    HOSTED_CLOUD_IDENTITY.register("default", identity);
    HOSTED_AUTH_STYLE.register("default", auth_style);
}

/// Register hosted-provider classification for one runtime context.
pub fn register_hosted_cloud_provider_policy_in(
    extensions: &crate::core::slots::Extensions,
    identity: HostedCloudIdentityClassifier,
    auth_style: HostedAuthStyleClassifier,
) {
    HOSTED_CLOUD_IDENTITY.register_in(extensions, "default", identity);
    HOSTED_AUTH_STYLE.register_in(extensions, "default", auth_style);
}

/// Whether the installed host owns this reserved cloud-provider identity.
pub fn is_hosted_cloud_identity(slug: &str) -> bool {
    HOSTED_CLOUD_IDENTITY
        .get("default")
        .is_some_and(|classify| classify(slug))
}

/// Whether the installed host owns this passive compatibility auth style.
pub fn is_hosted_auth_style(style: AuthStyle) -> bool {
    HOSTED_AUTH_STYLE
        .get("default")
        .is_some_and(|classify| classify(style))
}

/// Whether the selected runtime or process host owns this compatibility auth style.
pub(crate) fn is_hosted_auth_style_for(
    extensions: Option<&crate::core::slots::Extensions>,
    style: AuthStyle,
) -> bool {
    HOSTED_AUTH_STYLE
        .get_for(extensions, "default")
        .is_some_and(|classify| classify(style))
}

impl From<&tinyinference_providers::builtin::BuiltinCloudProvider> for BuiltinCloudProvider {
    fn from(preset: &tinyinference_providers::builtin::BuiltinCloudProvider) -> Self {
        Self {
            slug: preset.slug,
            label: preset.label,
            endpoint: preset.endpoint,
            auth_style: match preset.auth_style {
                BuiltinAuthStyle::Bearer => AuthStyle::Bearer,
                BuiltinAuthStyle::Anthropic => AuthStyle::Anthropic,
            },
        }
    }
}

#[path = "cloud_provider_presets.rs"]
mod extra_presets;
pub use extra_presets::EXTRA_CLOUD_PROVIDERS;

/// Every built-in preset: host-provided entries followed by the
/// provider-neutral catalog from `tinyinference-providers`.
pub fn builtin_cloud_providers() -> Vec<BuiltinCloudProvider> {
    let hosted = HOSTED_PRESETS
        .entries_context_first()
        .into_iter()
        .map(|(_, preset)| preset)
        .chain(EXTRA_CLOUD_PROVIDERS.iter().copied())
        .collect::<Vec<_>>();
    // Keep the product's managed entry first for settings presentation. The
    // owner catalog itself remains generic-first for exact-slug lookup.
    hosted
        .iter()
        .copied()
        .filter(|preset| {
            !UPSTREAM_BUILTINS
                .iter()
                .any(|generic| generic.slug == preset.slug)
        })
        .chain(UPSTREAM_BUILTINS.iter().map(BuiltinCloudProvider::from))
        .collect::<Vec<_>>()
}

/// Generic provider catalog extended with the host's registered preset
/// metadata. Host auth policy remains in the owning host registry.
pub fn builtin_presets() -> BuiltinPresets {
    let hosted = HOSTED_PRESETS
        .entries_context_first()
        .into_iter()
        .map(|(_, preset)| preset)
        .chain(EXTRA_CLOUD_PROVIDERS.iter().copied())
        .collect::<Vec<_>>();
    BuiltinPresets::new(hosted.iter().map(|preset| UpstreamBuiltinCloudProvider {
        slug: preset.slug,
        label: preset.label,
        endpoint: preset.endpoint,
        // Only route/catalog metadata is consumed from this generic catalog;
        // persisted OpenhumanJwt remains in the host's schema and is never
        // converted into runtime bearer authentication.
        auth_style: match preset.auth_style {
            AuthStyle::Anthropic => BuiltinAuthStyle::Anthropic,
            AuthStyle::Bearer | AuthStyle::OpenhumanJwt | AuthStyle::None => {
                BuiltinAuthStyle::Bearer
            }
        },
    }))
}

/// Looks up a provider preset, giving the generic catalog precedence over
/// host additions with the same slug.
pub fn builtin_cloud_provider(type_str: &str) -> Option<BuiltinCloudProvider> {
    if let Some(preset) = BuiltinPresets::default().builtin_cloud_provider(type_str) {
        return Some(BuiltinCloudProvider::from(preset));
    }
    let hosted = HOSTED_PRESETS
        .entries_context_first()
        .into_iter()
        .map(|(_, preset)| preset)
        .chain(EXTRA_CLOUD_PROVIDERS.iter().copied())
        .collect::<Vec<_>>();
    if let Some(preset) = hosted.iter().find(|preset| preset.slug == type_str) {
        return Some(*preset);
    }
    None
}

/// Whether a built-in cloud provider exposes the OpenAI **Responses API**
/// (`/v1/responses`).
///
/// OpenAI, the optional Prodex gateway, Nebius, OpenRouter and xAI serve `/responses`; other
/// built-ins (DeepSeek, Groq, Mistral, Fireworks, ...) are chat-completions-only.
/// Enabling the chat-completions-404 → `/responses` fallback for those
/// guarantees a second 404 against an endpoint that does not exist, which floods
/// Sentry with an empty-body `"<provider> Responses API error:"` event
/// (TAURI-RUST-5EN — same class as the local-provider TAURI-RUST-59Y fix). The
/// factory consults this to build chat-completions-only built-ins with
/// `new_no_responses_fallback`.
///
/// Custom / unknown slugs are intentionally NOT covered here (see
/// [`is_builtin_cloud_slug`]): a user-defined OpenAI-compatible endpoint may be
/// a genuine OpenAI proxy that does support `/responses`, so the factory keeps
/// the fallback for those.
pub fn builtin_cloud_supports_responses_api(slug: &str) -> bool {
    matches!(slug, "openai" | "prodex" | "nebius" | "openrouter" | "xai")
}

/// Extract the lowercased authority host from an endpoint URL, dropping the
/// scheme, any userinfo, the port, and the path. Returns `None` when no host
/// can be parsed. Tolerant of a missing scheme and of IPv6 literals.
fn endpoint_host(endpoint: &str) -> Option<String> {
    let s = endpoint.trim();
    // Drop the scheme (`https://…`); tolerate a bare `host/path` form.
    let after_scheme = s.split_once("://").map(|(_, rest)| rest).unwrap_or(s);
    // The authority ends at the first path / query / fragment delimiter.
    let authority = after_scheme
        .split(['/', '?', '#'])
        .next()
        .unwrap_or(after_scheme);
    // Strip any `user:pass@` userinfo prefix.
    let host_port = authority
        .rsplit_once('@')
        .map(|(_, host)| host)
        .unwrap_or(authority);
    // Strip the port, handling bracketed IPv6 literals (`[::1]:8080`).
    let host = if let Some(rest) = host_port.strip_prefix('[') {
        rest.split_once(']').map(|(h, _)| h).unwrap_or(rest)
    } else {
        host_port
            .rsplit_once(':')
            .map(|(h, _)| h)
            .unwrap_or(host_port)
    };
    let host = host.trim().to_ascii_lowercase();
    (!host.is_empty()).then_some(host)
}

/// Whether `host` is the authority host of any built-in cloud **inference**
/// provider (e.g. `openrouter.ai`, `api.openai.com`, `api.groq.com`).
///
/// Derived entirely from [`BUILTIN_CLOUD_PROVIDERS`] so the set stays in sync
/// with the provider registry. `host` is compared case-insensitively against
/// each preset's [`endpoint_host`]. Loopback addresses are excluded: local
/// gateways share their host with unrelated backends on other ports, so a
/// host alone cannot identify their service.
///
/// # Why this exists
///
/// `config.api_url` is overloaded: it is the chat/inference endpoint, but
/// `api::config::effective_backend_api_url` also reuses it as the
/// OpenHuman **backend** base for team/billing/auth calls. A BYO user who
/// points `api_url` at a provider's canonical base (`https://openrouter.ai/api/v1`)
/// would otherwise have every backend domain call routed to the inference host
/// → 400/404 (TAURI-RUST-HW1: 4932 team-usage 400s from `openrouter.ai`).
/// The backend-URL resolver uses this to treat such hosts as non-backend and
/// fall back to the default backend chain — the cloud analogue of the local-AI
/// guard that fixed the Ollama case (OPENHUMAN-TAURI-51/-80/-7Z).
pub fn host_is_builtin_cloud_provider(host: &str) -> bool {
    let host = host.trim().to_ascii_lowercase();
    if host.is_empty() || host_is_loopback(&host) {
        return false;
    }
    builtin_cloud_providers()
        .iter()
        .any(|p| endpoint_host(p.endpoint).as_deref() == Some(host.as_str()))
}

fn host_is_loopback(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host.ends_with(".localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

/// Whether an endpoint **host** is a known cloud host that does NOT serve the
/// OpenAI Responses API (`/v1/responses`) — i.e. it is chat-completions-only,
/// regardless of which user slug points at it.
///
/// Derived entirely from [`BUILTIN_CLOUD_PROVIDERS`]: a host is chat-only when
/// some built-in preset uses it AND no preset at that host advertises the
/// Responses API (including Nebius, OpenRouter, xAI, and Prodex). This closes the
/// custom-slug gap behind the builtin-slug gate
/// ([`builtin_cloud_supports_responses_api`]): a user slug pointed at, e.g.,
/// `integrate.api.nvidia.com` must never attempt `/responses` (TAURI-RUST-5A1),
/// while a genuinely unknown proxy host keeps the permissive fallback so a real
/// OpenAI proxy still gets `/responses`.
///
/// Loopback gateways are matched by URL origin, including scheme and port.
/// A chat-only service on one local port must not disable the fallback for
/// another local service, or inherit its Responses capability.
pub fn endpoint_host_is_chat_completions_only(endpoint: &str) -> bool {
    let Some(host) = endpoint_host(endpoint) else {
        return false;
    };
    let local_origin = if host_is_loopback(&host) {
        let Ok(url) = url::Url::parse(endpoint) else {
            return false;
        };
        Some(url.origin())
    } else {
        None
    };
    let mut matched_chat_only = false;
    for provider in builtin_cloud_providers().iter() {
        if endpoint_host(provider.endpoint).as_deref() == Some(host.as_str())
            && local_origin.as_ref().is_none_or(|origin| {
                url::Url::parse(provider.endpoint).is_ok_and(|preset| preset.origin() == *origin)
            })
        {
            if builtin_cloud_supports_responses_api(provider.slug) {
                // A Responses-capable built-in lives at this host → not chat-only.
                return false;
            }
            matched_chat_only = true;
        }
    }
    matched_chat_only
}

/// Authentication header style for a cloud provider.
///
/// Wire format is lowercase (e.g. `"bearer"`). Determines which HTTP headers
/// are attached when calling the provider's API.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum AuthStyle {
    /// OpenAI-compatible: `Authorization: Bearer <key>`
    #[default]
    Bearer,
    /// Anthropic: `x-api-key: <key>` + `anthropic-version: 2023-06-01`
    Anthropic,
    /// OpenHuman session JWT (injected by the backend provider, not stored here).
    OpenhumanJwt,
    /// No auth header — e.g. local Ollama.
    None,
}

impl AuthStyle {
    /// Parse the RPC compatibility spellings for a provider authentication style.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value.to_ascii_lowercase().as_str() {
            "bearer" => Ok(Self::Bearer),
            "anthropic" => Ok(Self::Anthropic),
            "openhuman_jwt" | "openhumanjwt" => Ok(Self::OpenhumanJwt),
            "none" => Ok(Self::None),
            other => Err(format!(
                "unknown auth_style '{}'; valid: bearer, anthropic, openhuman_jwt, none",
                other
            )),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bearer => "bearer",
            Self::Anthropic => "anthropic",
            Self::OpenhumanJwt => "openhuman_jwt",
            Self::None => "none",
        }
    }
}

/// How an explicit 2API-style provider returns tool calls. The native wire
/// format stays the default; QNML markup decoding requires user opt-in and
/// validates every call against the request's native tool declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ToolCallCodec {
    #[default]
    Native,
    Qnml,
    TaggedJson,
}
impl ToolCallCodec {
    pub fn is_native(&self) -> bool {
        *self == Self::Native
    }
}

/// Wire protocol selected for one concrete model. Auto preserves the
/// registry's existing Codex/Anthropic/Chat Completions routing. Overrides are
/// exact model-id matches, not provider-wide guesses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModelTransport {
    #[default]
    Auto,
    ChatCompletions,
    Responses,
}

/// Endpoint config for one cloud LLM provider.
///
/// **Note on secrets**: API keys are NOT stored on this struct. They live in
/// `auth-profiles.json` via `security::credentials::AuthService`,
/// keyed by `provider:<slug>` (falling back to bare `<slug>` for legacy
/// entries). The factory looks up the token at call time via
/// `inference::provider::factory::auth_key_for_slug`.
///
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(default)]
pub struct CloudProviderCreds {
    /// Opaque stable id, e.g. `"p_openai_a8c3f"`. Never shown in the UI.
    /// Generated once by [`generate_provider_id`] and never changes.
    pub id: String,
    /// Routing key chosen by the user or seeded from the legacy type.
    /// Lower-case alphanumeric + `-`. Must be unique per config and not in the
    /// reserved list (see [`is_slug_reserved`]). The factory resolves
    /// `"<slug>:<model>"` strings against this field.
    pub slug: String,
    /// Human-readable display label, supplied by the frontend. Not used in routing.
    pub label: String,
    /// OpenAI-compatible base URL (`/models`, `/chat/completions` etc. are appended).
    pub endpoint: String,
    /// Authentication header style.
    pub auth_style: AuthStyle,

    /// Optional alternate OpenAI-compatible base URL for messages containing image input.
    /// This is a provider-group default; model_image_endpoints overrides it per model id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_endpoint: Option<String>,

    /// Per-model image/vision endpoints, keyed by the unmodified upstream model id.
    /// Not credentials: every request continues to use this provider's scoped AuthService key.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub model_image_endpoints: std::collections::HashMap<String, String>,

    /// Optional selection strategy: round_robin (default), least_busy or priority_weighted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_strategy: Option<String>,
    /// Opt-in parsing for Qwen/QNML text-encoded tool calls exposed by some
    /// independent 2API gateways. Never alters native function-call output.
    #[serde(default, skip_serializing_if = "ToolCallCodec::is_native")]
    pub tool_call_codec: ToolCallCodec,

    /// Explicit per-model transport. Unlisted model ids retain Auto routing;
    /// no metadata or free-tier provider list can silently flip protocol.
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub model_transports: std::collections::HashMap<String, ModelTransport>,

    /// Default provider-native model used by BYOK routes without a model suffix.
    // keep until v0.66 (review after 2027-01-11): active BYOK routes and the
    // inference settings RPC depend on this field; replacement needs route migration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
}

impl Default for CloudProviderCreds {
    fn default() -> Self {
        Self {
            id: String::new(),
            slug: String::new(),
            label: String::new(),
            endpoint: String::new(),
            auth_style: AuthStyle::Bearer,
            image_endpoint: None,
            model_image_endpoints: std::collections::HashMap::new(),
            rotation_strategy: None,
            tool_call_codec: ToolCallCodec::Native,
            model_transports: std::collections::HashMap::new(),
            default_model: None,
        }
    }
}

impl CloudProviderCreds {
    /// The exact model's configured protocol, or registry behavior for
    /// unknown models. No provider-name or endpoint heuristics are applied.
    pub fn model_transport(&self, model: &str) -> ModelTransport {
        self.model_transports
            .get(model)
            .copied()
            .unwrap_or_default()
    }
}

/// Normalize model transport overrides received over RPC. Model ids are exact
/// upstream wire ids, so silently trimming a key could route a different model.
pub fn normalize_model_transports(
    overrides: std::collections::HashMap<String, ModelTransport>,
) -> Result<std::collections::HashMap<String, ModelTransport>, String> {
    if overrides.len() > 512 {
        return Err("too many model transport overrides (maximum 512)".into());
    }
    for id in overrides.keys() {
        if id.is_empty() || id.len() > 200 || id.trim() != id || id.chars().any(char::is_control) {
            return Err("model transport id must be 1-200 printable, unpadded characters".into());
        }
    }
    Ok(overrides)
}

/// Validate OpenAI-specific protocol routing against the credential wire format.
/// Auto retains native Anthropic and managed routes; explicit protocol paths
/// require an OpenAI-compatible Bearer or unauthenticated backend.
pub fn normalize_model_transports_for_auth(
    overrides: std::collections::HashMap<String, ModelTransport>,
    auth_style: AuthStyle,
) -> Result<std::collections::HashMap<String, ModelTransport>, String> {
    let normalized = normalize_model_transports(overrides)?;
    if !matches!(auth_style, AuthStyle::Bearer | AuthStyle::None)
        && normalized
            .values()
            .any(|mode| *mode != ModelTransport::Auto)
    {
        return Err(
            "explicit OpenAI model transports require bearer or none authentication".into(),
        );
    }
    Ok(normalized)
}

/// Normalize settings provided over either model-settings RPC surface.
/// Reject embedded URL credentials, unsupported schemes and duplicate trimmed ids.
pub type NormalizedFleetConfig = (
    Option<String>,
    std::collections::HashMap<String, String>,
    Option<String>,
);

pub fn normalize_fleet_config(
    image_endpoint: Option<String>,
    model_image_endpoints: std::collections::HashMap<String, String>,
    rotation_strategy: Option<String>,
) -> Result<NormalizedFleetConfig, String> {
    fn image_url(raw: &str) -> Result<String, String> {
        let value = raw.trim();
        // Do not rely on WHATWG normalization for credential-bearing routes:
        // it may silently discard embedded CR/LF/TAB and change the URL.
        // The frontend uses the same explicit control-character boundary.
        if value.chars().any(char::is_control) {
            return Err("image endpoint contains embedded control characters".into());
        }
        let url = reqwest::Url::parse(value)
            .map_err(|_| "image endpoint must be an absolute HTTP(S) URL".to_string())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(
                "image endpoint must be HTTP(S), with no credentials, query or fragment".into(),
            );
        }
        // Model Fleet may reuse the configured provider's API key for image
        // routes. Sending it to a remote plain-HTTP image endpoint would leak
        // the credential in transit. Explicit loopback development gateways
        // keep working, including IPv6 localhost.
        if url.scheme() == "http" {
            let host = url.host_str().unwrap_or_default().trim_matches(['[', ']']);
            let loopback = host.eq_ignore_ascii_case("localhost")
                || host
                    .parse::<std::net::IpAddr>()
                    .is_ok_and(|address| address.is_loopback());
            if !loopback {
                return Err("image endpoints must use HTTPS outside loopback hosts".to_string());
            }
        }
        Ok(value.to_owned())
    }

    let group = image_endpoint
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(image_url)
        .transpose()?;
    if model_image_endpoints.len() > 512 {
        return Err("too many image model endpoint overrides (maximum 512)".into());
    }
    let mut routes = std::collections::HashMap::new();
    for (id, raw_url) in model_image_endpoints {
        let id = id.trim();
        if id.is_empty() || id.len() > 200 || id.chars().any(char::is_control) {
            return Err("image model id must be 1-200 printable characters".into());
        }
        let url = image_url(&raw_url)?;
        if routes.insert(id.to_owned(), url).is_some() {
            return Err(format!("duplicate image model id: {id}"));
        }
    }
    let strategy = rotation_strategy
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|raw| {
            if matches!(raw, "round_robin" | "least_busy" | "priority_weighted") {
                Ok(raw.to_string())
            } else {
                Err("unknown key rotation strategy".to_string())
            }
        })
        .transpose()?;
    Ok((group, routes, strategy))
}

/// Reserved slugs that may not be used for user-configured providers.
/// These are sentinels in the factory's routing grammar.
///
/// `ollama` is deliberately NOT reserved: the AI settings panel registers an
/// `ollama` `cloud_providers` entry so `list_configured_models` can resolve
/// the user's chosen base_url for the model dropdown. The factory's chat
/// routing is unaffected — the `ollama:<model>` prefix branch in
/// `factory::create_chat_provider_from_string` fires before the
/// `<slug>:<model>` cloud-provider lookup, so a synthetic `ollama` entry
/// never reaches `make_cloud_provider_by_slug`. When no `cloud_providers`
/// row exists (config drift, upgrade from a build that only persisted
/// `config.local_ai.base_url`, flush-vs-probe race),
/// `inference::provider::models::catalog_listing::list_configured_models`
/// falls back to a synthetic entry via `synthesize_local_runtime_entry`
/// (Sentry TAURI-RUST-28Z fix). The same fallback applies to `lmstudio`.
pub fn is_slug_reserved(s: &str) -> bool {
    matches!(s.trim(), "" | "cloud" | "openhuman" | "pid")
}

/// Generate a short opaque id for a new provider entry.
///
/// Format: `"p_<slug>_<5 random alphanumerics>"`, e.g. `"p_openai_a8c3f"`.
/// The random suffix is not cryptographically strong — it only needs to be
/// unique within a single user's config file.
pub fn generate_provider_id(slug: &str) -> String {
    super::generate_provider_id_with_prefix("p", slug)
}

#[cfg(test)]
#[path = "cloud_providers_tests.rs"]
mod tests;
