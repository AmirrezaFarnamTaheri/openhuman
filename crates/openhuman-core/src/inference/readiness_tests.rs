use super::*;
use crate::core::runtime::{CoreContext, DomainSet};
use crate::core::slots::Extensions;
use crate::inference::provider::factory::PROVIDER_OPENHUMAN;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tempfile::TempDir;

#[tokio::test]
async fn cached_probe_serves_definitive_negative_results() {
    let _guard = crate::inference::inference_test_guard_async().await;
    let _exclusive = crate::inference::provider::factory::test_provider_override::exclusive();
    let _signed_in = crate::platform::llm_gate::SignedOutTestGuard::set(false);
    let tmp = TempDir::new().unwrap();
    let config = Config {
        workspace_dir: tmp.path().join("workspace"),
        action_dir: tmp.path().join("workspace"),
        config_path: tmp.path().join("config.toml"),
        ..Config::default()
    };
    let hit_count = Arc::new(AtomicUsize::new(0));
    let counter = hit_count.clone();
    let mut provider =
        crate::inference::registry_test_support::default_managed_entry(PROVIDER_OPENHUMAN)
            .expect("managed test provider");
    // The host owns the managed provider and its readiness endpoint. Exercise
    // the core's cache through that registration seam, without assuming the
    // core still embeds a particular HTTP backend.
    provider.probe = Some(Arc::new(move |role, _, _| {
        assert_eq!(role, "summarization");
        counter.fetch_add(1, Ordering::SeqCst);
        Box::pin(async { Err("API key not configured for provider".to_string()) })
    }));
    let extensions = Extensions::default();
    crate::inference::registry::register_chat_model_in(&extensions, PROVIDER_OPENHUMAN, provider);
    let context = CoreContext::for_test_with_extensions(
        DomainSet::full(),
        Some(config.workspace_dir.clone()),
        extensions,
    );

    CoreContext::scope(context, async {
        let ctx = DispatchCtx::ambient();
        let first = cached_probe(&ctx, "summarization", &config).await;
        assert_eq!(
            first,
            Err("API key not configured for provider".to_string())
        );
        assert_eq!(hit_count.load(Ordering::SeqCst), 1);
        let second = cached_probe(&ctx, "summarization", &config).await;
        assert_eq!(second, first);
        assert_eq!(hit_count.load(Ordering::SeqCst), 1);
    })
    .await;
}
