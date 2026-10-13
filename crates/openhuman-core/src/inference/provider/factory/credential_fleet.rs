//! Per-call credential selection and multimodal endpoint routing for cloud LLMs.
//!
//! A fleet wraps native models, never copies network adapters. Every call picks a
//! healthy credential and retries only failures known to be safe to reroute.
//! A live stream is NEVER replayed after emitting content; its terminal event
//! still updates the pool and drop releases the key lease.
use std::sync::Arc;

use async_trait::async_trait;
use tinyinference_llm::message::{ContentBlock, Message};
use tinyinference_llm::model::{
    ChatModel, InputModality, InputSource, ModelProfile, ModelRequest, ModelResponse,
    ModelStream, ModelStreamItem, ProviderError,
};

use super::key_pool::{self, KeyFailure, RotationStrategy};

/// Detect actual image input (not merely a vision-capable model).
pub(super) fn request_has_image(request: &ModelRequest) -> bool {
    request.messages.iter().any(|message| {
        let blocks: &[ContentBlock] = match message {
            Message::User(m) => &m.content,
            Message::Assistant(m) => &m.content,
            Message::Tool(m) => &m.content,
            _ => &[],
        };
        blocks.iter().any(|block| matches!(block, ContentBlock::Image(_)))
    })
}

#[derive(Clone, Copy)]
struct RetryDecision {
    failure: KeyFailure,
    after_ms: Option<u64>,
}
fn classify(status: Option<u16>, code: Option<&str>, retry_after_ms: Option<u64>) -> Option<RetryDecision> {
    let failure = match status {
        Some(401) => KeyFailure::Authentication,
        Some(429) => KeyFailure::RateLimited,
        Some(403) if code.is_some_and(|c| {
            let c = c.to_ascii_lowercase();
            c.contains("quota") || c.contains("rate_limit") || c.contains("resource_exhausted")
        }) => KeyFailure::RateLimited,
        _ => return None,
    };
    Some(RetryDecision { failure, after_ms: retry_after_ms })
}
fn provider_decision(error: &ProviderError) -> Option<RetryDecision> {
    classify(error.status, error.code.as_deref(), error.retry_after_ms)
}
fn model_decision(error: &tinyinference_llm::Error) -> Option<RetryDecision> {
    match error {
        tinyinference_llm::Error::Provider(e) => provider_decision(e),
        _ => None,
    }
}

struct Lease {
    scope: String,
    key: String,
}
impl Lease {
    fn new(scope: &str, key: &str) -> Self {
        key_pool::acquire(scope, key);
        Self { scope: scope.to_owned(), key: key.to_owned() }
    }
    fn record(&self, failure: Option<RetryDecision>) {
        key_pool::record_result_for(
            &self.scope, &self.key,
            failure.map(|failure| failure.failure),
            failure.and_then(|failure| failure.after_ms),
        );
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        key_pool::release(&self.scope, &self.key);
    }
}

/// Key-index alignment is fixed within this instance. The shared key_pool uses
/// raw secret identity for outcomes so reconfiguration in a parallel instance
/// cannot corrupt a different credential's health.
pub(super) struct CredentialFleetModel {
    scope: String,
    keys: Vec<String>,
    models: Vec<Arc<dyn ChatModel<()>>>,
    image_models: Option<Vec<Arc<dyn ChatModel<()>>>>,
    strategy: RotationStrategy,
}
impl CredentialFleetModel {
    pub(super) fn new(
        scope: String,
        keys: Vec<String>,
        models: Vec<Arc<dyn ChatModel<()>>>,
        image_models: Option<Vec<Arc<dyn ChatModel<()>>>>,
        strategy: RotationStrategy,
    ) -> Self {
        assert!(!models.is_empty() && models.len() == keys.len());
        assert!(image_models.as_ref().is_none_or(|m| m.len() == models.len()));
        Self { scope, keys, models, image_models, strategy }
    }
    fn route(&self, idx: usize, image: bool) -> &Arc<dyn ChatModel<()>> {
        if image {
            if let Some(ref group) = self.image_models {
                return &group[idx];
            }
        }
        &self.models[idx]
    }
}
#[async_trait]
impl ChatModel<()> for CredentialFleetModel {
    fn profile(&self) -> Option<&ModelProfile> {
        self.models.first().and_then(|model| model.profile())
    }
    fn supports_input(&self, modality: InputModality, mime: &str, source: InputSource) -> bool {
        let models = self.image_models.as_ref().unwrap_or(&self.models);
        models.first().is_some_and(|model| model.supports_input(modality, mime, source))
    }
    fn cache_identity(&self) -> Option<String> {
        self.models.first().and_then(|model| model.cache_identity())
    }
    async fn invoke(&self, state: &(), request: ModelRequest) -> tinyinference_llm::Result<ModelResponse> {
        let image = request_has_image(&request);
        let mut tried = Vec::new();
        let mut last_error = None;
        while let Some((index, key)) =
            key_pool::choose_key(&self.scope, &self.keys, &tried, self.strategy)
        {
            tried.push(index);
            let lease = Lease::new(&self.scope, &key);
            match self.route(index, image).invoke(state, request.clone()).await {
                Ok(result) => {
                    lease.record(None);
                    return Ok(result);
                }
                Err(error) => {
                    let decision = model_decision(&error);
                    lease.record(Some(decision.unwrap_or(RetryDecision {
                        failure: KeyFailure::Other, after_ms: None,
                    })));
                    if decision.is_none() { return Err(error); }
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| tinyinference_llm::Error::Model(
            "No available API keys for this provider; check rate limits or replace invalid keys".to_string()
        )))
    }
    async fn stream(&self, state: &(), request: ModelRequest) -> tinyinference_llm::Result<ModelStream> {
        let image = request_has_image(&request);
        let mut tried = Vec::new();
        let mut last_error = None;
        while let Some((index, key)) =
            key_pool::choose_key(&self.scope, &self.keys, &tried, self.strategy)
        {
            tried.push(index);
            let lease = Lease::new(&self.scope, &key);
            match self.route(index, image).stream(state, request.clone()).await {
                Ok(stream) => {
                    // The lease is owned by this stream's mapping closure.
                    // It releases automatically on completion *or cancellation*.
                    let mut recorded = false;
                    return Ok(stream.map_items(move |item| {
                        if !recorded {
                            match &item {
                                ModelStreamItem::Completed(_) => {
                                    lease.record(None);
                                    recorded = true;
                                }
                                ModelStreamItem::ProviderFailed(error) => {
                                    lease.record(provider_decision(error).or(Some(RetryDecision {
                                        failure: KeyFailure::Other, after_ms: None,
                                    })));
                                    recorded = true;
                                }
                                ModelStreamItem::Failed(_) => {
                                    lease.record(Some(RetryDecision { failure: KeyFailure::Other, after_ms: None }));
                                    recorded = true;
                                }
                                _ => {}
                            }
                        }
                        item
                    }));
                }
                Err(error) => {
                    let decision = model_decision(&error);
                    lease.record(Some(decision.unwrap_or(RetryDecision {
                        failure: KeyFailure::Other, after_ms: None,
                    })));
                    if decision.is_none() { return Err(error); }
                    last_error = Some(error);
                }
            }
        }
        Err(last_error.unwrap_or_else(|| tinyinference_llm::Error::Model(
            "No available API keys for this provider; check rate limits or replace invalid keys".to_string()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tinyinference_llm::message::{ImageRef, UserMessage};
    #[test]
    fn image_endpoints_only_apply_when_request_contains_images() {
        let text = ModelRequest { messages: vec![Message::user("text")], ..Default::default() };
        assert!(!request_has_image(&text));
        let image = ModelRequest {
            messages: vec![Message::User(UserMessage {
                content: vec![
                    ContentBlock::Text("What's here?".into()),
                    ContentBlock::Image(ImageRef { url: "data:image/png;base64,AAAA".into(), mime_type: Some("image/png".into()) }),
                ],
            })],
            ..Default::default()
        };
        assert!(request_has_image(&image));
    }
    #[test]
    fn provider_failures_are_rerouted_conservatively() {
        assert_eq!(classify(Some(401), None, None).unwrap().failure, KeyFailure::Authentication);
        assert_eq!(classify(Some(429), None, Some(5000)).unwrap().failure, KeyFailure::RateLimited);
        assert_eq!(classify(Some(403), Some("quota_exhausted"), None).unwrap().failure, KeyFailure::RateLimited);
        assert!(classify(Some(400), None, None).is_none());
        assert!(classify(Some(403), Some("forbidden"), None).is_none());
    }
}
