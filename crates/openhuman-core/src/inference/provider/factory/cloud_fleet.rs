//! Minimal native-cloud adapter for multi-key BYOK requests.
//! The ordinary single-key model stays unchanged. Inspection constructs only
//! the first credential's model; it must not perform runtime failover.
use std::sync::Arc;

use tinyinference_llm::model::ChatModel;

use super::credential_fleet::CredentialFleetModel;
use super::key_pool::RotationStrategy;

pub(super) fn build_cloud_model_with_keys(
    scope: String,
    keys: Vec<String>,
    enable_rotation: bool,
    mut build: impl FnMut(&str) -> anyhow::Result<Arc<dyn ChatModel<()>>>,
) -> anyhow::Result<Arc<dyn ChatModel<()>>> {
    let keys = if keys.is_empty() {
        vec![String::new()]
    } else {
        keys
    };
    if !enable_rotation || keys.len() == 1 {
        return build(&keys[0]);
    }

    let models = keys
        .iter()
        .map(|key| build(key))
        .collect::<anyhow::Result<Vec<_>>>()?;
    Ok(Arc::new(CredentialFleetModel::new(
        scope,
        keys,
        models,
        None,
        RotationStrategy::RoundRobin,
    )))
}

#[cfg(test)]
#[path = "cloud_fleet_tests.rs"]
mod tests;
