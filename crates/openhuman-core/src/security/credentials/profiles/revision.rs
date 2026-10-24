//! Process-local cache revisions for attempted profile mutations.
//! No credential material is retained here. A fresh process has no warm clients.
//! Writes from another process are not observed by this in-process tracker.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, Weak};

// A weak reference keeps this allocation's identity unique without keeping the
// backend (connections, files or workers) alive after replacement.
struct BackendIdentity(Weak<dyn crate::storage::StorageBackend>);

impl PartialEq for BackendIdentity {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for BackendIdentity {}
impl Hash for BackendIdentity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.0.as_ptr() as *const () as usize).hash(state);
    }
}

#[derive(Hash, PartialEq, Eq)]
enum StoreKey {
    File(PathBuf),
    BackendScope(BackendIdentity, String),
}

fn revisions() -> &'static Mutex<HashMap<StoreKey, uuid::Uuid>> {
    static REVISIONS: OnceLock<Mutex<HashMap<StoreKey, uuid::Uuid>>> = OnceLock::new();
    REVISIONS.get_or_init(Default::default)
}

fn store_key(state_dir: &Path) -> Result<StoreKey, crate::storage::StorageError> {
    let backend = crate::storage::installed();
    let scope = backend
        .as_ref()
        .map(|_| crate::storage::current_scope())
        .transpose()?;
    Ok(key_for_scope(
        state_dir,
        backend
            .as_ref()
            .zip(scope.as_ref())
            .map(|(backend, scope)| (backend, scope.as_str())),
    ))
}

fn key_for_scope(
    state_dir: &Path,
    scope: Option<(&Arc<dyn crate::storage::StorageBackend>, &str)>,
) -> StoreKey {
    match scope {
        Some((backend, scope)) => {
            StoreKey::BackendScope(BackendIdentity(Arc::downgrade(backend)), scope.to_owned())
        }
        None => StoreKey::File(
            state_dir
                .canonicalize()
                .unwrap_or_else(|_| state_dir.to_path_buf()),
        ),
    }
}

pub(crate) fn profile_revision(state_dir: &Path) -> Option<uuid::Uuid> {
    let key = match store_key(state_dir) {
        Ok(key) => key,
        // An unresolved scope must never reuse an existing provider client.
        Err(_) => return Some(uuid::Uuid::new_v4()),
    };
    Some(
        *revisions()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .entry(key)
            .or_insert_with(uuid::Uuid::new_v4),
    )
}

pub(super) fn profiles_saved(state_dir: &Path) {
    let Ok(key) = store_key(state_dir) else {
        return;
    };
    revisions()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .insert(key, uuid::Uuid::new_v4());
}

/// Invalidate after all attempted writes, including partial keychain success
/// followed by metadata failure. Dropping on an error is intentional.
pub(super) struct ProfileMutation<'a>(pub(super) &'a Path);

impl Drop for ProfileMutation<'_> {
    fn drop(&mut self) {
        profiles_saved(self.0);
    }
}

#[cfg(test)]
#[path = "revision_tests.rs"]
mod tests;
