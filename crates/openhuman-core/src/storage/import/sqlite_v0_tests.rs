//! Real-driver regression tests for upgrading SQLite databases from TSD v0.
//! Every test uses a private database and never touches the operator profile.
use super::*;
use crate::storage::{CollectionSpec, Scope};

fn legacy_db(dir: &std::path::Path, broken: bool) -> std::path::PathBuf {
    let db = dir.join("storage").join("storage.db");
    std::fs::create_dir_all(db.parent().unwrap()).unwrap();
    let conn = rusqlite::Connection::open(&db).unwrap();
    conn.execute_batch(
        "CREATE TABLE _tsd_docs (scope TEXT, coll TEXT, id TEXT, version INTEGER, doc TEXT);
         CREATE TABLE _tsd_tombstones (scope TEXT, coll TEXT, id TEXT, version INTEGER);
         CREATE TABLE _tsd_streams (scope TEXT, stream TEXT, seq INTEGER, value TEXT);
         CREATE TABLE _tsd_stream_meta (scope TEXT, stream TEXT, base INTEGER, len INTEGER, sealed INTEGER);
         CREATE TABLE _tsd_blobs (scope TEXT, key TEXT, content_type TEXT, bytes BLOB, expires_at_ms INTEGER);
         CREATE TABLE _tsd_collections (coll TEXT PRIMARY KEY, spec TEXT NOT NULL) WITHOUT ROWID;
         CREATE VIRTUAL TABLE _tsd_fts USING fts5(scope UNINDEXED, coll UNINDEXED, id UNINDEXED, body);"
    ).unwrap();
    conn.execute(
        "INSERT INTO _tsd_collections (coll, spec) VALUES (?1, ?2)",
        rusqlite::params![
            "migrated",
            serde_json::to_string(&CollectionSpec::new("migrated")).unwrap()
        ],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _tsd_docs VALUES ('local', 'migrated', 'one', 7, ?1)",
        [if broken {
            "{broken".to_owned()
        } else {
            serde_json::json!({"value": "preserved", "created_at": "2024-05-01T12:30:00Z"})
                .to_string()
        }],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _tsd_tombstones VALUES ('local','migrated','deleted',9)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _tsd_streams VALUES ('local','events',0,'{\"event\":1}')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _tsd_stream_meta VALUES ('local','events',0,1,1)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _tsd_blobs VALUES ('local','binary','application/octet-stream',x'010203',NULL)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO _tsd_fts (scope,coll,id,body) VALUES ('local','migrated','one','preserved')",
        [],
    )
    .unwrap();
    db
}

#[test]
fn a_current_sqlite_database_never_creates_legacy_migration_lockfiles() {
    let dir = tempfile::tempdir().unwrap();
    let db = dir.path().join("storage.db");
    let conn = Connection::open(&db).unwrap();
    conn.execute_batch("CREATE TABLE docs (id TEXT PRIMARY KEY, body TEXT)")
        .unwrap();
    drop(conn);

    upgrade_if_needed(&db).unwrap();

    assert!(
        !dir.path().join("storage.db.v0-upgrade.lock").exists(),
        "a current DB must not require a legacy migration lock or write to disk at startup"
    );
}

#[test]
fn a_stalled_peer_cannot_hold_legacy_migration_queued_forever() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), false);
    let original = std::fs::read(&db).unwrap();
    let _peer = MigrationLock::acquire(&db).unwrap();
    let result = MigrationLock::acquire_with_timeout(
        &db,
        std::time::Duration::from_millis(60),
    );
    let error = match result {
        Ok(_) => panic!("the waiting process must not bypass the peer's migration lock"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("another process"), "{error}");
    assert!(error.contains("retry"), "{error}");
    assert_eq!(std::fs::read(&db).unwrap(), original);
}

#[test]
fn concurrent_upgrade_process_handles_are_serialized_before_database_inspection() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), false);
    let first = MigrationLock::acquire(&db).expect("own the migration lock");

    let (tx, rx) = std::sync::mpsc::channel();
    let contender_db = db.clone();
    let contender = std::thread::spawn(move || {
        let _second = MigrationLock::acquire(&contender_db)
            .expect("acquire the same database migration lock after first releases it");
        tx.send(()).unwrap();
    });

    assert!(
        rx.recv_timeout(std::time::Duration::from_millis(75)).is_err(),
        "a second process handle must not enter migration while the first holds the lock"
    );
    drop(first);
    rx.recv_timeout(std::time::Duration::from_secs(3))
        .expect("second migration acquires the lock after the first exits");
    contender.join().unwrap();
}

#[tokio::test]
async fn old_sqlite_upgrades_before_backend_opens_and_retains_an_original_backup() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), false);
    let url = format!("sqlite:{}", dir.path().join("storage").display());
    let backend = crate::storage::open(&url).await.unwrap();
    let docs = backend
        .for_scope(&Scope::local())
        .unwrap()
        .documents()
        .clone();
    let record = docs.get("migrated", "one").await.unwrap().unwrap();
    assert_eq!(record.doc["value"], "preserved");
    assert_eq!(record.version, tinystoragedrivers_types::Version(7));
    assert!(docs.get("migrated", "deleted").await.unwrap().is_none());
    let conn = rusqlite::Connection::open(&db).unwrap();
    assert!(!table_exists(&conn, "_tsd_docs").unwrap());
    assert!(table_exists(&conn, "docs").unwrap());
    // TSD v2 compares stored CREATE statements, including quoted identifiers.
    // Merely retaining the v0 collection table under its original spelling
    // looks structurally valid but makes the v2 driver reject every open.
    let declarations: String = conn
        .query_row(
            "SELECT sql FROM sqlite_schema WHERE type='table' AND name='_tsd_collections'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(
        declarations.contains("\"_tsd_collections\""),
        "upgraded collection DDL must use the v2 driver's quoted identifier"
    );
    let row_count: i64 = conn
        .query_row("SELECT count(*) FROM blobs WHERE key = 'binary'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(row_count, 1);
    let stream_count: i64 = conn
        .query_row(
            "SELECT count(*) FROM streams WHERE name = 'events'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(stream_count, 1);
    let backups: Vec<_> = std::fs::read_dir(db.parent().unwrap())
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains("pre-v2-")
        })
        .collect();
    assert_eq!(backups.len(), 1, "upgrade must retain one rollback copy");
    let old = rusqlite::Connection::open(&backups[0]).unwrap();
    assert!(table_exists(&old, "_tsd_docs").unwrap());
    let reopened = crate::storage::open(&url).await.unwrap();
    let value = reopened
        .for_scope(&Scope::local())
        .unwrap()
        .documents()
        .get("migrated", "one")
        .await
        .unwrap();
    assert!(value.is_some(), "reopening must not repeat the migration");
}

#[tokio::test]
async fn bad_legacy_document_rolls_back_without_destroying_user_data() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), true);
    let bytes_before = std::fs::read(&db).unwrap();
    let result = crate::storage::open(&format!("sqlite:{}", db.display())).await;
    assert!(
        result.is_err(),
        "invalid historical rows must not be silently dropped"
    );
    let conn = rusqlite::Connection::open(&db).unwrap();
    assert!(
        table_exists(&conn, "_tsd_docs").unwrap(),
        "conversion rolled back"
    );
    assert!(
        !table_exists(&conn, "docs").unwrap(),
        "no half-created v2 tables"
    );
    let historical: String = conn
        .query_row("SELECT doc FROM _tsd_docs WHERE id='one'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(historical, "{broken");
    assert!(!bytes_before.is_empty());
}

// Windows FlushFileBuffers requires GENERIC_WRITE. A verified VACUUM INTO
// rollback file must be durably flushed before we touch the original DB.
// Reproduces the File::open(...).sync_all() access-denied startup failure.
#[cfg(windows)]
#[test]
fn windows_migration_preserves_and_flushes_the_verified_rollback_backup() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), false);
    let conn = rusqlite::Connection::open_with_flags(&db, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .expect("open the historical database");
    let counts = inventory(&conn).expect("inventory the source");

    let path = backup(&conn, &db, &counts)
        .expect("flush a validated VACUUM INTO backup using a write-capable file handle");

    let verified = rusqlite::Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .expect("open the verified backup");
    check_integrity(&verified).expect("backup integrity");
    assert_eq!(inventory(&verified).unwrap(), counts);
    assert!(table_exists(&verified, "_tsd_docs").unwrap());
}

#[test]
fn large_profile_upgrade_refuses_unbudgeted_backup_and_wal_growth() {
    // A 12 GiB legacy file with 34.8 GiB free must not start a backup that
    // could exhaust the system disk halfway through the conversion.
    const GIB: u64 = 1 << 30;
    let legacy_bytes = 12_909_268_992_u64;
    let available = 34_u64 * GIB + (8_u64 * GIB / 10);
    let required = minimum_migration_headroom(legacy_bytes);
    assert!(required > available);
    assert!(migration_headroom_shortfall(legacy_bytes, available).is_some());
    assert_eq!(migration_headroom_shortfall(legacy_bytes, required), None);
}

#[test]
fn upgrade_headroom_calculation_never_overflows_for_untrusted_file_size() {
    assert_eq!(minimum_migration_headroom(u64::MAX), u64::MAX);
    assert!(migration_headroom_shortfall(u64::MAX, 10).is_some());
}
