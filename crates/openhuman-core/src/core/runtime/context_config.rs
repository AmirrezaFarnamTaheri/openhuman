use std::sync::Arc;

use super::{ContextOverlay, CoreContext};

pub(super) fn record_context_config_overlay(
    parent_profile: Option<&str>,
    parent_config: Option<&crate::config::Config>,
    overlay: &mut ContextOverlay,
) {
    let config_overlay = if let Some(profile_id) = overlay.profile.as_deref() {
        let parent_profile_config = (parent_profile == Some(profile_id))
            .then_some(parent_config)
            .flatten();
        Some(
            crate::config::schema::load::context_overlay::ConfigContextOverlay::derive_profile(
                parent_profile_config,
                &overlay.config,
            ),
        )
    } else {
        parent_config.map(|parent_config| {
            crate::config::schema::load::context_overlay::ConfigContextOverlay::derive(
                parent_config,
                &overlay.config,
            )
        })
    };
    if let Some(config_overlay) = config_overlay {
        match config_overlay {
            Ok(config_overlay) => {
                overlay.config.runtime_state.context_overlay =
                    std::sync::Mutex::new(Some(config_overlay));
            }
            Err(error) => {
                log::warn!("[core-context] unable to record derived config overrides: {error}")
            }
        }
    }
}

impl CoreContext {
    /// The embedder-supplied config for the current dispatch, if there is one.
    ///
    /// The read path for `config::ops::load_config_with_timeout`.
    pub fn current_embedder_config() -> Option<crate::config::Config> {
        Self::current().and_then(|ctx| ctx.latest_embedder_config())
    }

    pub(crate) fn latest_embedder_config(&self) -> Option<crate::config::Config> {
        // The library embedder's initial config exists even before its mutable
        // snapshot is installed. Do not lose the caller's configuration just
        // because it did not ask for live profile refresh.
        let Some(snapshot) = self.latest_embedder_config.as_ref() else {
            return self.embedder_config.clone();
        };
        let mut config = snapshot.lock().ok()?.clone();
        config.runtime_state.context_snapshot = Some(Arc::downgrade(snapshot));
        Some(config)
    }

    /// Read the embedder-supplied config for the current dispatch without
    /// cloning it. Prefer this over [`Self::current_embedder_config`] when only
    /// a field or a predicate is needed: a `Config` clone is a large copy.
    pub fn with_current_embedder_config<R>(
        f: impl FnOnce(&crate::config::Config) -> R,
    ) -> Option<R> {
        Self::current().and_then(|ctx| {
            if let Some(snapshot) = &ctx.latest_embedder_config {
                let config = snapshot.lock().ok()?;
                Some(f(&config))
            } else {
                ctx.embedder_config.as_ref().map(f)
            }
        })
    }
}
