use super::*;
#[cfg(feature = "storage-sqlite")]
use crate::storage::layout::legacy;
#[cfg(feature = "storage-sqlite")]
use crate::storage::layout::test_fixtures::{fixture_docs, v1_profile};
use crate::storage::layout::Target;
use tempfile::TempDir;

#[cfg(all(windows, feature = "storage-sqlite"))]
#[test]
fn windows_layout_database_copy_flushes_before_publication() {
    // Win32 FlushFileBuffers rejects the read-only File::open handle. A valid
    // snapshot must be durably flushed and published without touching its
    // historical source.
    let fx = v1_profile();
    let source = legacy::v1_db(&fx.target);
    let original = std::fs::read(&source).expect("read v1 database");
    let dest = fx.target.storage_db();

    let result = copy(&source, &dest)
        .expect("VACUUM INTO copy flushes on Windows and is renamed into place");
    assert!(!result.reused);
    assert_eq!(table_counts(&dest).unwrap(), table_counts(&source).unwrap());
    assert_eq!(std::fs::read(&source).unwrap(), original);
    assert!(!dest.with_file_name("storage.db.tmp").exists());
}


#[test]
#[cfg(feature = "storage-sqlite")]
fn the_copy_has_the_sources_counts_and_leaves_the_source_alone() {
    let fx = v1_profile();
    let source = legacy::v1_db(&fx.target);
    checkpoint(&source).unwrap();
    let before = std::fs::read(&source).unwrap();
    let dest = fx.target.storage_db();
    let copied = copy(&source, &dest).unwrap();
    assert!(!copied.reused);
    assert_eq!(copied.counts, table_counts(&source).unwrap());
    assert_eq!(copied.counts, table_counts(&dest).unwrap());
    assert!(!dest.with_file_name("storage.db.tmp").exists());
    assert_eq!(
        std::fs::read(&source).unwrap(),
        before,
        "the source is read-only to the copy"
    );
    assert_eq!(fixture_docs(&dest).len(), 3);
}

#[test]
#[cfg(feature = "storage-sqlite")]
fn a_matching_copy_is_kept_and_a_stale_one_replaced() {
    let fx = v1_profile();
    let source = legacy::v1_db(&fx.target);
    let dest = fx.target.storage_db();
    copy(&source, &dest).unwrap();
    assert!(copy(&source, &dest).unwrap().reused);
    // a leftover that does not match the source is rebuilt
    std::fs::remove_file(&dest).unwrap();
    let stale = Connection::open(&dest).unwrap();
    stale.execute_batch("CREATE TABLE junk (x)").unwrap();
    drop(stale);
    let again = copy(&source, &dest).unwrap();
    assert!(!again.reused);
    assert_eq!(table_counts(&dest).unwrap(), table_counts(&source).unwrap());
}

#[test]
fn a_stale_copy_with_equal_row_counts_is_never_reused() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("legacy.db");
    let dest = dir.path().join("storage.db");
    let conn = Connection::open(&source).unwrap();
    conn.execute_batch(
        "CREATE TABLE records (id TEXT PRIMARY KEY, body TEXT NOT NULL);
         INSERT INTO records VALUES ('one', 'correct-value');",
    )
    .unwrap();
    drop(conn);

    assert!(!copy(&source, &dest).unwrap().reused);

    // Simulate a previously interrupted migration leaving a same-size, stale
    // destination. Its row inventory matches, but its document bytes do not.
    Connection::open(&dest)
        .unwrap()
        .execute(
            "UPDATE records SET body = 'wrong-value' WHERE id = 'one'",
            [],
        )
        .unwrap();
    assert_eq!(table_counts(&source).unwrap(), table_counts(&dest).unwrap());

    let refreshed = copy(&source, &dest).unwrap();
    assert!(!refreshed.reused, "row counts alone must not authorize reuse");
    let value: String = Connection::open(&dest)
        .unwrap()
        .query_row("SELECT body FROM records WHERE id = 'one'", [], |row| row.get(0))
        .unwrap();
    assert_eq!(value, "correct-value");
}

#[test]
fn a_copy_with_matching_rows_but_changed_schema_is_not_reused() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("legacy.db");
    let dest = dir.path().join("storage.db");
    Connection::open(&source)
        .unwrap()
        .execute_batch(
            "CREATE TABLE records (id TEXT PRIMARY KEY, body TEXT NOT NULL);
             INSERT INTO records VALUES ('one', 'correct');",
        )
        .unwrap();
    copy(&source, &dest).unwrap();

    Connection::open(&dest)
        .unwrap()
        .execute_batch("CREATE INDEX stale_index ON records(body)")
        .unwrap();
    assert_eq!(table_counts(&source).unwrap(), table_counts(&dest).unwrap());

    assert!(!copy(&source, &dest).unwrap().reused);
    let stale_index: i64 = Connection::open(&dest)
        .unwrap()
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE name = 'stale_index'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(stale_index, 0, "recovered schema must match the source");
}

#[test]
fn a_count_mismatch_names_the_table() {
    let a: Counts = [("docs".to_owned(), 3), ("blobs".to_owned(), 1)].into();
    let b: Counts = [("docs".to_owned(), 2), ("blobs".to_owned(), 1)].into();
    let diff = mismatch(&a, &b).unwrap();
    assert!(diff.contains("docs"), "{diff}");
    assert!(!diff.contains("blobs"), "{diff}");
    assert!(mismatch(&a, &a).is_none());
}

#[test]
fn a_missing_source_is_an_error() {
    let dir = TempDir::new().unwrap();
    let target = Target::new(dir.path(), "u1");
    assert!(copy(&dir.path().join("nope.db"), &target.storage_db()).is_err());
}

#[test]
fn inventory_and_copy_leave_a_live_wal_source_unchanged() {
    let dir = TempDir::new().unwrap();
    let source = dir.path().join("legacy.db");
    let dest = dir.path().join("copy.db");
    let writer = Connection::open(&source).unwrap();
    writer.pragma_update(None, "journal_mode", "WAL").unwrap();
    writer.pragma_update(None, "wal_autocheckpoint", 0).unwrap();
    writer
        .execute_batch("CREATE TABLE records (id TEXT); INSERT INTO records VALUES ('one');")
        .unwrap();
    let wal = source.with_file_name("legacy.db-wal");
    assert!(
        wal.is_file(),
        "fixture must retain committed rows in its WAL"
    );

    let db_before = std::fs::read(&source).unwrap();
    let wal_before = std::fs::read(&wal).unwrap();
    let counts = table_counts(&source).unwrap();
    assert_eq!(counts.get("records"), Some(&1));
    copy(&source, &dest).unwrap();
    assert_eq!(std::fs::read(&source).unwrap(), db_before);
    assert_eq!(std::fs::read(&wal).unwrap(), wal_before);
    drop(writer);
}
