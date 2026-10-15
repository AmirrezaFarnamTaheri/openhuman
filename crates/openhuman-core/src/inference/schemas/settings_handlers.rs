//! Handlers for model and provider settings: resolution, status, client
//! config, model and local-runtime settings updates, model listing, device
//! auth errors, and diagnostics.

use serde::de::Deserializer;
use serde::Deserialize;
use serde_json::{Map, Value};

use super::{deserialize_params, to_json};
use crate::config::rpc as config_rpc;
use crate::core::all::ControllerFuture;
use crate::core::Outcome;

#[derive(Debug, Deserialize)]
pub(super) struct InferenceResolveModelParams {
    hint: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct InferenceModelRouteUpdate {
    hint: String,
    model: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct InferenceCloudProviderUpdate {
    id: Option<String>,
    slug: String,
    #[serde(default)]
    label: Option<String>,
    endpoint: String,
    #[serde(default)]
    auth_style: Option<String>,
    /// Alternate image/vision base URL for this provider.
    #[serde(default)]
    image_endpoint: Option<String>,
    /// Model-id-specific image URL overrides, keyed within the provider group.
    #[serde(default)]
    model_image_endpoints: std::collections::HashMap<String, String>,
    /// Credential strategy: round_robin | least_busy | priority_weighted.
    #[serde(default)]
    rotation_strategy: Option<String>,
    #[serde(default)]
    tool_call_codec: crate::config::schema::cloud_providers::ToolCallCodec,
    #[serde(default)]
    model_transports: std::collections::HashMap<
        String,
        crate::config::schema::cloud_providers::ModelTransport,
    >,
    #[serde(default)]
    ca_cert_pem: Option<String>,
    // keep until v0.66 (review after 2027-01-11): incoming RPC discriminator
    // remains accepted until shipped-client parameter coverage exists.
    #[serde(rename = "type", default)]
    legacy_type: Option<String>,
    #[serde(default)]
    default_model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct InferenceUpdateModelSettingsParams {
    api_url: Option<String>,
    inference_url: Option<String>,
    api_key: Option<String>,
    default_model: Option<String>,
    default_temperature: Option<f64>,
    model_routes: Option<Vec<InferenceModelRouteUpdate>>,
    cloud_providers: Option<Vec<InferenceCloudProviderUpdate>>,
    #[serde(default)]
    model_registry: Option<Vec<crate::config::schema::ModelRegistryEntry>>,
    primary_cloud: Option<String>,
    chat_provider: Option<String>,
    reasoning_provider: Option<String>,
    agentic_provider: Option<String>,
    coding_provider: Option<String>,
    vision_provider: Option<String>,
    memory_provider: Option<String>,
    embeddings_provider: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct InferenceUpdateLocalSettingsParams {
    runtime_enabled: Option<bool>,
    opt_in_confirmed: Option<bool>,
    provider: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present_json")]
    base_url: Option<Value>,
    model_id: Option<String>,
    chat_model_id: Option<String>,
    usage_embeddings: Option<bool>,
    api_key: Option<String>,
}

#[derive(Debug, Deserialize)]
pub(super) struct InferenceListModelsParams {
    provider_id: String,
    /// Optional one-shot BYOK candidate; never saved to the credential store.
    #[serde(default)]
    candidate_api_key: Option<String>,
    #[serde(default)]
    candidate_provider: Option<crate::config::schema::cloud_providers::CloudProviderCreds>,
    #[serde(default)]
    candidate_ca_cert_pem: Option<String>,
}

pub(super) fn handle_inference_resolve_model(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let p = deserialize_params::<InferenceResolveModelParams>(params)?;
        let config = config_rpc::load_config_with_timeout().await?;
        let resolved =
            crate::inference::provider::factory::resolve_model_for_hint(&p.hint, &config);
        // Whether the resolved model accepts image input — drives the chat UI's
        // image-attachment affordance. Managed OpenHuman tiers consult the
        // core-owned per-tier map, which returns `true` for the reasoning and
        // vision tiers (and their `hint:` aliases) and `false` for every other
        // tier; custom/BYOK models are covered by the user's per-model
        // `model_registry.vision` flag.
        let vision = crate::inference::model_context::model_supports_vision(&resolved, &config);
        to_json(Outcome::new(
            serde_json::json!({ "model": resolved, "vision": vision }),
            vec![],
        ))
    })
}

pub(super) fn handle_inference_status(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        to_json(crate::inference::rpc::inference_status(&config).await?)
    })
}

pub(super) fn handle_inference_get_client_config(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move { to_json(crate::inference::rpc::inference_get_client_config().await?) })
}

#[derive(Debug, Deserialize)]
struct ModelFleetResetParams {
    slug: String,
}

/// Model-fleet health and cooldowns are process-local, keyed by workspace.
/// The response contains only counts and statuses, never raw token material.
pub(super) fn handle_inference_model_fleet_status(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        let status = crate::inference::provider::factory::model_fleet_health(&config);
        to_json(Outcome::new(status, vec![]))
    })
}

pub(super) fn handle_inference_model_fleet_reset(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let update = deserialize_params::<ModelFleetResetParams>(params)?;
        let config = config_rpc::load_config_with_timeout().await?;
        crate::inference::provider::factory::model_fleet_reset(&config, &update.slug)?;
        to_json(Outcome::new(
            serde_json::json!({ "reset": true, "provider": update.slug }),
            vec![],
        ))
    })
}

pub(super) fn handle_inference_update_model_settings(
    params: Map<String, Value>,
) -> ControllerFuture {
    Box::pin(async move {
        let update = deserialize_params::<InferenceUpdateModelSettingsParams>(params)?;
        let ca_certs = update
            .cloud_providers
            .as_ref()
            .map(|providers| {
                crate::config::ops::collect_provider_ca_certs(
                    providers
                        .iter()
                        .map(|provider| (provider.slug.as_str(), provider.ca_cert_pem.as_deref())),
                )
            })
            .transpose()?;
        let patch = config_rpc::ModelSettingsPatch {
            api_url: update.api_url,
            inference_url: update.inference_url,
            api_key: update.api_key,
            default_model: update.default_model,
            default_temperature: update.default_temperature,
            cloud_provider_ca_certs: ca_certs,
            model_routes: update.model_routes.map(|routes| {
                routes
                    .into_iter()
                    .map(|route| crate::config::ModelRouteConfig {
                        hint: route.hint,
                        model: route.model,
                    })
                    .collect()
            }),
            cloud_providers: update
                .cloud_providers
                .map(|entries| {
                    use crate::config::schema::cloud_providers::{
                        generate_provider_id, is_slug_reserved, builtin_cloud_provider, AuthStyle,
                        CloudProviderCreds,
                    };
                    let reserved_count = entries
                        .iter()
                        .filter(|e| {
                            let t = e.slug.trim();
                            !t.is_empty() && is_slug_reserved(t)
                        })
                        .count();
                    if reserved_count > 0 {
                        log::debug!(
                            "[inference] update_model_settings: dropping {} reserved cloud provider slug(s)",
                            reserved_count
                        );
                    }
                    entries
                        .into_iter()
                        // Silently drop entries whose (non-empty) slug is reserved —
                        // typically the migration-seeded "openhuman" / "cloud" /
                        // "pid" built-ins that the frontend echoes back on every
                        // save (see `migrations::unify_ai_provider_settings`).
                        // Empty slugs still fall through so the explicit
                        // validation error below fires for actual frontend
                        // bugs. `apply_model_settings` re-injects the existing
                        // reserved entries from the stored config so they
                        // aren't dropped on save.
                        .filter(|entry| {
                            let trimmed = entry.slug.trim();
                            trimmed.is_empty() || !is_slug_reserved(trimmed)
                        })
                        .map(|entry| {
                            let slug = entry.slug.trim().to_string();
                            if slug.is_empty() {
                                return Err("cloud provider slug must not be empty".to_string());
                            }
                            let legacy_provider = entry.legacy_type.as_deref()
                                .map(str::trim)
                                .and_then(builtin_cloud_provider);
                            let legacy_auth = legacy_provider
                                .map(|provider| provider.auth_style.as_str())
                                .unwrap_or("bearer");
                            let endpoint = if entry.endpoint.is_empty() {
                                legacy_provider.map(|provider| provider.endpoint.to_string())
                                    .unwrap_or(entry.endpoint)
                            } else {
                                entry.endpoint
                            };
                            let auth_style = match entry
                                .auth_style
                                .as_deref()
                                .unwrap_or(legacy_auth)
                                .to_ascii_lowercase()
                                .as_str()
                            {
                                "bearer" => AuthStyle::Bearer,
                                "anthropic" => AuthStyle::Anthropic,
                                "openhuman_jwt" | "openhumanjwt" => AuthStyle::OpenhumanJwt,
                                "none" => AuthStyle::None,
                                other => {
                                    return Err(format!(
                                        "unknown auth_style '{}'; valid: bearer, anthropic, openhuman_jwt, none",
                                        other
                                    ))
                                }
                            };
                            let id = entry
                                .id
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or_else(|| generate_provider_id(&slug));
                            let label = entry
                                .label
                                .filter(|s| !s.trim().is_empty())
                                .unwrap_or_else(|| slug.clone());
                            let (image_endpoint, model_image_endpoints, rotation_strategy) =
                                crate::config::schema::cloud_providers::normalize_fleet_config(
                                    entry.image_endpoint,
                                    entry.model_image_endpoints,
                                    entry.rotation_strategy,
                                )?;
                            let provider = CloudProviderCreds {
                                id,
                                slug,
                                label,
                                endpoint,
                                auth_style,
                                image_endpoint,
                                model_image_endpoints,
                                rotation_strategy,
                                tool_call_codec: entry.tool_call_codec,
                                model_transports:
                                    crate::config::schema::cloud_providers::normalize_model_transports_for_auth(
                                        entry.model_transports,
                                        auth_style,
                                    )?,
                                default_model: entry.default_model,
                            };
                            Ok(provider)
                        })
                        .collect::<Result<Vec<_>, String>>()
                })
                .transpose()?,
            model_registry: update.model_registry,
            primary_cloud: update.primary_cloud,
            chat_provider: update.chat_provider,
            reasoning_provider: update.reasoning_provider,
            agentic_provider: update.agentic_provider,
            coding_provider: update.coding_provider,
            vision_provider: update.vision_provider,
            memory_provider: update.memory_provider,
            embeddings_provider: update.embeddings_provider,
        };
        to_json(crate::inference::rpc::inference_update_model_settings(patch).await?)
    })
}

pub(super) fn handle_inference_update_local_settings(
    params: Map<String, Value>,
) -> ControllerFuture {
    Box::pin(async move {
        let update = deserialize_params::<InferenceUpdateLocalSettingsParams>(params)?;
        let base_url = match update.base_url {
            None => None,
            Some(Value::Null) => Some(None),
            Some(Value::String(value)) => Some(Some(value)),
            Some(_) => return Err("invalid params: base_url must be a string or null".to_string()),
        };
        let patch = config_rpc::LocalAiSettingsPatch {
            runtime_enabled: update.runtime_enabled,
            opt_in_confirmed: update.opt_in_confirmed,
            provider: update.provider,
            base_url,
            model_id: update.model_id,
            chat_model_id: update.chat_model_id,
            usage_embeddings: update.usage_embeddings,
            api_key: update.api_key,
        };
        to_json(crate::inference::rpc::inference_update_local_settings(patch).await?)
    })
}

pub(super) fn handle_inference_list_models(params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let request = deserialize_params::<InferenceListModelsParams>(params)?;
        if request.candidate_api_key.is_some() || request.candidate_provider.is_some() {
            return to_json(
                crate::inference::provider::ops::list_configured_models_with_candidate_key(
                    &request.provider_id,
                    request.candidate_api_key.as_deref().unwrap_or(""),
                    request.candidate_provider,
                    request.candidate_ca_cert_pem,
                )
                .await?,
            );
        }
        to_json(crate::inference::rpc::inference_list_models(&request.provider_id).await?)
    })
}

pub(super) fn handle_inference_provider_auth_errors(
    _params: Map<String, Value>,
) -> ControllerFuture {
    Box::pin(async move { to_json(crate::inference::rpc::inference_provider_auth_errors().await?) })
}

pub(super) fn handle_inference_diagnostics(_params: Map<String, Value>) -> ControllerFuture {
    Box::pin(async move {
        let config = config_rpc::load_config_with_timeout().await?;
        to_json(crate::inference::rpc::inference_diagnostics(&config).await?)
    })
}

fn deserialize_present_json<'de, D>(deserializer: D) -> Result<Option<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    Value::deserialize(deserializer).map(Some)
}

#[cfg(test)]
mod tool_codec_update_contract_tests {
    use super::*;
    use crate::config::schema::cloud_providers::ToolCallCodec;

    #[test]
    fn rpc_tool_codec_field_keeps_json_tagged_or_qnml_selection() {
        for (wire, expected) in [
            ("native", ToolCallCodec::Native),
            ("qnml", ToolCallCodec::Qnml),
            ("tagged_json", ToolCallCodec::TaggedJson),
        ] {
            let input: InferenceCloudProviderUpdate = serde_json::from_value(
                serde_json::json!({
                    "slug": "peer-gateway",
                    "endpoint": "https://example.net/v1",
                    "tool_call_codec": wire,
                }),
            )
            .expect("valid codec name");
            assert_eq!(input.tool_call_codec, expected);
        }
    }

    #[test]
    fn older_provider_update_payload_defaults_to_native_tool_calls() {
        let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
            "slug": "legacy",
            "endpoint": "https://example.net/v1"
        }))
        .unwrap();
        assert_eq!(input.tool_call_codec, ToolCallCodec::Native);
        assert!(input.model_transports.is_empty());
    }

    #[test]
    fn rpc_model_transports_preserve_exact_model_ids_and_protocols() {
        use crate::config::schema::cloud_providers::{normalize_model_transports, ModelTransport};

        let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
            "slug": "mixed-gateway",
            "endpoint": "https://example.net/v1",
            "model_transports": {
                "vendor/responses-v2": "responses",
                "legacy-chat": "chat_completions",
                "default": "auto"
            }
        }))
        .expect("valid wire transport selection");

        let routes = normalize_model_transports(input.model_transports).unwrap();
        assert_eq!(routes["vendor/responses-v2"], ModelTransport::Responses);
        assert_eq!(routes["legacy-chat"], ModelTransport::ChatCompletions);
        assert_eq!(routes["default"], ModelTransport::Auto);
    }

    #[test]
    fn rpc_model_transports_reject_invalid_ids_before_persistence() {
        use crate::config::schema::cloud_providers::normalize_model_transports;

        let input: InferenceCloudProviderUpdate = serde_json::from_value(serde_json::json!({
            "slug": "mixed-gateway",
            "endpoint": "https://example.net/v1",
            "model_transports": {" padded-model ": "responses"}
        }))
        .expect("serde permits the key; normalization validates it");
        assert!(normalize_model_transports(input.model_transports).is_err());
    }
}
