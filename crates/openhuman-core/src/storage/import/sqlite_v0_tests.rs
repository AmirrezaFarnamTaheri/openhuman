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
         CREATE TABLE _tsd_collections (coll TEXT PRIMARY KEY, spec TEXT);
         CREATE VIRTUAL TABLE _tsd_fts USING fts5(scope UNINDEXED, coll UNINDEXED, id UNINDEXED, body);"
    ).unwrap();
    conn.execute(
        "INSERT INTO _tsd_collections (coll, spec) VALUES (?1, ?2)",
        rusqlite::params!["migrated", serde_json::to_string(&CollectionSpec::new("migrated")).unwrap()],
    ).unwrap();
    conn.execute(
        "INSERT INTO _tsd_docs VALUES ('local', 'migrated', 'one', 7, ?1)",
        [if broken { "{broken".to_owned() } else { serde_json::json!({"value": "preserved", "created_at": "2024-05-01T12:30:00Z"}).to_string() }],
    ).unwrap();
    conn.execute("INSERT INTO _tsd_tombstones VALUES ('local','migrated','deleted',9)", []).unwrap();
    conn.execute("INSERT INTO _tsd_streams VALUES ('local','events',0,'{\"event\":1}')", []).unwrap();
    conn.execute("INSERT INTO _tsd_stream_meta VALUES ('local','events',0,1,1)", []).unwrap();
    conn.execute("INSERT INTO _tsd_blobs VALUES ('local','binary','application/octet-stream',x'010203',NULL)", []).unwrap();
    conn.execute("INSERT INTO _tsd_fts (scope,coll,id,body) VALUES ('local','migrated','one','preserved')", []).unwrap();
    db
}

#[tokio::test]
async fn old_sqlite_upgrades_before_backend_opens_and_retains_an_original_backup() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), false);
    let url = format!("sqlite:{}", dir.path().join("storage").display());
    let backend = crate::storage::open(&url).await.unwrap();
    let docs = backend.for_scope(&Scope::local()).unwrap().documents().clone();
    let record = docs.get("migrated", "one").await.unwrap().unwrap();
    assert_eq!(record.doc["value"], "preserved");
    assert_eq!(record.version, 7);
    assert!(docs.get("migrated", "deleted").await.unwrap().is_none());
    let conn = rusqlite::Connection::open(&db).unwrap();
    assert!(!table_exists(&conn, "_tsd_docs").unwrap());
    assert!(table_exists(&conn, "docs").unwrap());
    let row_count: i64 = conn.query_row("SELECT count(*) FROM blobs WHERE key = 'binary'", [], |r| r.get(0)).unwrap();
    assert_eq!(row_count, 1);
    let stream_count: i64 = conn.query_row("SELECT count(*) FROM streams WHERE name = 'events'", [], |r| r.get(0)).unwrap();
    assert_eq!(stream_count, 1);
    let backups: Vec<_> = std::fs::read_dir(db.parent().unwrap()).unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.file_name().unwrap().to_string_lossy().contains("pre-v2-")).collect();
    assert_eq!(backups.len(), 1, "upgrade must retain one rollback copy");
    let old = rusqlite::Connection::open(&backups[0]).unwrap();
    assert!(table_exists(&old, "_tsd_docs").unwrap());
    let reopened = crate::storage::open(&url).await.unwrap();
    let value = reopened.for_scope(&Scope::local()).unwrap().documents().get("migrated", "one").await.unwrap();
    assert!(value.is_some(), "reopening must not repeat the migration");
}

#[tokio::test]
async fn bad_legacy_document_rolls_back_without_destroying_user_data() {
    let dir = tempfile::tempdir().unwrap();
    let db = legacy_db(dir.path(), true);
    let bytes_before = std::fs::read(&db).unwrap();
    let result = crate::storage::open(&format!("sqlite:{}", db.display())).await;
    assert!(result.is_err(), "invalid historical rows must not be silently dropped");
    let conn = rusqlite::Connection::open(&db).unwrap();
    assert!(table_exists(&conn, "_tsd_docs").unwrap(), "conversion rolled back");
    assert!(!table_exists(&conn, "docs").unwrap(), "no half-created v2 tables");
    let historical: String = conn.query_row("SELECT doc FROM _tsd_docs WHERE id='one'", [], |r| r.get(0)).unwrap();
    assert_eq!(historical, "{broken");
    assert!(!bytes_before.is_empty());
}
