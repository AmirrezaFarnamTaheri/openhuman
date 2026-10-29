//! Shared plumbing for the keyring backends that sit over
//! `tinystoragedrivers`' `SecretStore` port: error mapping, UTF-8 decoding
//! and the desktop corruption policy.

use std::path::Path;

use tinystoragedrivers::secrets::{EncryptedFileSecrets, SecretStore as _};
use tinystoragedrivers::{ErrorKind, StorageError};
use zeroize::Zeroizing;

use crate::security::secret_storage::error::SecretStorageError;
use crate::security::secret_storage::file_store;

/// Map a file-secret driver error into a redacted, actionable diagnostic.
///
/// The pinned driver keeps native I/O details in its `source`, not its
/// Display message. Do not print that source: it can contain paths or
/// credential material. For legacy Windows lock failures, expose only the
/// stable OS error category/code so support can distinguish ACL problems
/// from a simultaneous writer without guessing or deleting secrets.
pub(super) fn backend_error(error: StorageError) -> SecretStorageError {
    let guidance = if error.message() == "could not open the lock file" {
        let code = std::error::Error::source(&error)
            .and_then(|source| source.downcast_ref::<std::io::Error>())
            .and_then(std::io::Error::raw_os_error);
        match code {
            #[cfg(windows)]
            Some(5) => " — Windows access denied (Win32 5); check the existing lock file's owner and permissions; preserve your encrypted secrets and key files",
            #[cfg(windows)]
            Some(32) => " — Windows sharing violation (Win32 32); close other OpenHuman instances and retry; preserve your encrypted secrets and key files",
            _ => " — secret-store lock unavailable; do not delete the lock or key files; preserve your encrypted secrets and retry after checking the file permissions",
        }
    } else {
        ""
    };
    SecretStorageError::Backend(format!("secret store: {error}{guidance}"))
}

/// Retry only a *temporary Windows lock-sharing collision*, not an ACL
/// denial or corrupt secret store. All attempts execute the normal guarded
/// driver operation; none remove, reset, quarantine or bypass any lock.
pub(super) fn retry_windows_secret_lock<T>(
    mut operation: impl FnMut() -> tinystoragedrivers::Result<T>,
) -> tinystoragedrivers::Result<T> {
    #[cfg(windows)]
    {
        for retry in 0..3 {
            match operation() {
                Err(error) if is_windows_sharing_violation(&error) => {
                    std::thread::sleep(std::time::Duration::from_millis(40_u64 << retry));
                }
                result => return result,
            }
        }
    }
    operation()
}

#[cfg(windows)]
fn is_windows_sharing_violation(error: &StorageError) -> bool {
    if error.message() != "could not open the lock file" {
        return false;
    }
    let code = std::error::Error::source(error)
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .and_then(std::io::Error::raw_os_error);
    matches!(code, Some(32 | 33))
}

/// Whether the driver reports the secrets file itself as unreadable
/// (undecryptable under this key, or not the expected JSON).
pub(super) fn is_corruption(error: &StorageError) -> bool {
    matches!(error.kind(), ErrorKind::Crypto | ErrorKind::Serialization)
}

/// Decode a stored value as UTF-8 text.
pub(super) fn utf8(
    key: &str,
    value: Option<Zeroizing<Vec<u8>>>,
) -> Result<Option<String>, SecretStorageError> {
    value
        .map(|bytes| {
            String::from_utf8(bytes.to_vec()).map_err(|source| SecretStorageError::InvalidUtf8 {
                key: key.to_string(),
                source,
            })
        })
        .transpose()
}

/// Desktop policy for a `secrets.enc` the driver could not read: move it
/// aside and let the caller continue on an empty store.
///
/// The driver fails closed (error, file untouched), which suits a server with
/// an operator but would wedge a desktop: every write fails, so the user can
/// never sign in again. Today's behaviour is kept instead: quarantine the
/// bytes (`secrets.enc.corrupt.<ts>`, never deleted) and log loudly.
///
/// The driver's probe takes the cross-process lock used by writers. A
/// fingerprint before that probe is compared again under OpenHuman's lock
/// before quarantine, so a concurrent writer's replacement stays untouched.
///
/// # Errors
///
/// When the file cannot be moved aside. Continuing would overwrite secrets
/// that could not be read, so the operation fails closed instead.
pub(super) fn recover_corrupt_file(
    path: &Path,
    key: &[u8; 32],
    cause: &StorageError,
) -> Result<(), SecretStorageError> {
    recover_corrupt_file_after_probe(path, key, cause, || {})
}

fn recover_corrupt_file_after_probe(
    path: &Path,
    key: &[u8; 32],
    cause: &StorageError,
    after_probe: impl FnOnce(),
) -> Result<(), SecretStorageError> {
    let before_probe = file_store::fingerprint(path)?;
    let store = EncryptedFileSecrets::at_path(path, Zeroizing::new(*key));
    match crate::storage::block_on(async move { store.list("").await }) {
        Ok(_) => {
            log::info!("[keyring:encrypted_file] secrets file readable again; not quarantining");
            return Ok(());
        }
        // Still corrupt: fall through to quarantine.
        Err(error) if is_corruption(&error) => {}
        // Anything else (a transient read or permission error) says nothing
        // about the file's contents: never move a possibly healthy file aside.
        Err(error) => return Err(backend_error(error)),
    }
    after_probe();

    // The driver holds this same sidecar lock while probing. Acquire
    // OpenHuman's guard only after its probe returns, then ensure a concurrent
    // writer did not replace the corrupt bytes with a healthy store.
    let _guard = file_store::lock_for_write(path)?;
    if file_store::fingerprint(path)? != before_probe {
        log::info!(
            "[keyring:encrypted_file] secrets file changed after the failed probe; not quarantining"
        );
        return Ok(());
    }
    log::error!(
        "[keyring:encrypted_file] secrets file unreadable ({}): master key may have changed \
         or the file is corrupt",
        cause.message()
    );
    if file_store::quarantine_corrupt(path, "enc").is_none() {
        return Err(SecretStorageError::Backend(
            "secrets file is unreadable and could not be moved aside".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
#[path = "adapter_tests.rs"]
pub(crate) mod tests;
