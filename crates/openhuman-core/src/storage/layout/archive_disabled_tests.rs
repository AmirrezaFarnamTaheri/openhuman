use super::*;

#[test]
fn a_lean_host_rejects_archive_creation_without_touching_disk() {
    let temp = tempfile::tempdir().unwrap();
    let archive = temp.path().join("migration.tar.zst");
    let error = write(&archive, "local", 0, &[]).unwrap_err().to_string();
    assert!(error.contains("storage-sqlite"), "{error}");
    assert!(!archive.exists(), "unsupported archive must not be created");
}

#[test]
fn a_lean_host_does_not_pretend_to_verify_or_restore_archives() {
    let temp = tempfile::tempdir().unwrap();
    let archive = temp.path().join("migration.tar.zst");
    let verify_err = verify(&archive).unwrap_err().to_string();
    let restore_err = restore(&archive, temp.path()).unwrap_err().to_string();
    assert!(verify_err.contains("storage-sqlite"), "{verify_err}");
    assert!(restore_err.contains("storage-sqlite"), "{restore_err}");
}
