//! Crash-safe upgrade of TSD v0 SQLite databases at the host boundary.
//!
//! TinyStorageDrivers v2 deliberately refuses the old _tsd_docs layout.
//! Run this importer before opening a driver, retaining a complete backup and
//! performing all schema changes in one SQLite transaction. This changes
//! database format, not profile/workspace layout.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::storage::rusqlite::types::Value as SqlValue;
use crate::storage::rusqlite::{self, params, Connection, OpenFlags};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tinystoragedrivers::CollectionSpec;
use tinystoragedrivers_types::value;

use crate::storage::StorageError;

const OLD_TABLES: &[&str] = &[
    "_tsd_docs",
    "_tsd_tombstones",
    "_tsd_fts",
    "_tsd_streams",
    "_tsd_stream_meta",
    "_tsd_blobs",
    "_tsd_collections",
];

/// Reserve space for a full VACUUM INTO rollback image, newly copied v2
/// tables and transaction journal/WAL. This is a conservative *lower-bound
/// preflight*, not a guarantee of success for arbitrary index/data layouts.
fn minimum_migration_headroom(source_bytes: u64) -> u64 {
    const MARGIN: u64 = 256 * 1024 * 1024;
    source_bytes.saturating_mul(3).saturating_add(MARGIN)
}

fn migration_headroom_shortfall(source_bytes: u64, free_bytes: u64) -> Option<u64> {
    let required = minimum_migration_headroom(source_bytes);
    (free_bytes < required).then_some(required)
}

/// Fail before VACUUM INTO, which would otherwise fill a nearly full system
/// disk and leave an incomplete backup. Never delete user data to make room.
fn preflight_migration_disk_space(file: &Path) -> Result<(), StorageError> {
    let source_bytes = std::fs::metadata(file)
        .map_err(|e| error("inspect source database size", e))?
        .len();
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    let available =
        fs2::available_space(parent).map_err(|e| error("inspect available disk space", e))?;
    if let Some(required) = migration_headroom_shortfall(source_bytes, available) {
        const GIB: f64 = 1_073_741_824.0;
        return Err(StorageError::backend(format!(
            "legacy SQLite migration needs at least {:.2} GiB free for a verified backup,              copied tables and SQLite journal; only {:.2} GiB is available.              Preserve storage.db and the entire workspace, then free space or              migrate the workspace to a larger drive before retrying",
            required as f64 / GIB,
            available as f64 / GIB
        )));
    }
    Ok(())
}

fn database_path(path: &Path) -> PathBuf {
    if path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("db") || s.eq_ignore_ascii_case("sqlite"))
    {
        path.to_path_buf()
    } else {
        path.join("storage.db")
    }
}

/// Coordinates the v0→v2 format cut across desktop, CLI and other
/// processes. SQLite's BEGIN IMMEDIATE alone is too late: two processes
/// could both read v0, create rollback snapshots, and then race the DDL.
struct MigrationLock {
    _handle: std::fs::File,
}

impl MigrationLock {
    fn acquire(database: &Path) -> Result<Self, StorageError> {
        // A different instance can keep a lock while performing a legitimate
        // long-running conversion. Wait, but never block a startup indefinitely.
        Self::acquire_with_timeout(database, Duration::from_secs(8 * 60))
    }

    fn acquire_with_timeout(database: &Path, maximum: Duration) -> Result<Self, StorageError> {
        let mut lock_name = database
            .file_name()
            .unwrap_or_default()
            .to_os_string();
        lock_name.push(".v0-upgrade.lock");
        let lock_path = database.with_file_name(lock_name);
        let mut options = std::fs::OpenOptions::new();
        options.create(true).read(true).write(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt as _;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        // A lockfile is not a user database and is intentionally retained:
        // removing it while another client holds it would split the lock
        // namespace and defeat cross-process coordination.
        let lock = options
            .open(&lock_path)
            .map_err(|e| error("open migration coordination lock", e))?;
        if !lock
            .metadata()
            .map_err(|e| error("inspect migration coordination lock", e))?
            .is_file()
        {
            return Err(StorageError::backend(
                "legacy SQLite migration coordination lock is not a file",
            ));
        }
        let started = Instant::now();
        loop {
            match fs2::FileExt::try_lock_exclusive(&lock) {
                Ok(()) => break,
                Err(e)
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.raw_os_error() == Some(33) =>
                {
                    if started.elapsed() >= maximum {
                        return Err(StorageError::backend(
                            "legacy SQLite upgrade is still locked by another process; close the other OpenHuman instance and retry. Your database and backups were left unchanged",
                        ));
                    }
                    std::thread::sleep(
                        Duration::from_millis(100).min(maximum.saturating_sub(started.elapsed())),
                    );
                }
                Err(e) => return Err(error("acquire migration coordination lock", e)),
            }
        }
        Ok(Self { _handle: lock })
    }
}

fn error(context: &str, source: impl std::fmt::Display) -> StorageError {
    StorageError::backend(format!("legacy SQLite migration ({context}): {source}"))
}

fn table_exists(conn: &Connection, name: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1)",
        [name],
        |row| row.get(0),
    )
}

fn has_column(conn: &Connection, table: &str, column: &str) -> rusqlite::Result<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM pragma_table_info(?1) WHERE name=?2)",
        params![table, column],
        |row| row.get(0),
    )
}

fn check_integrity(conn: &Connection) -> Result<(), StorageError> {
    let result: String = conn
        .query_row("PRAGMA quick_check", [], |row| row.get(0))
        .map_err(|e| error("integrity check", e))?;
    if result == "ok" {
        Ok(())
    } else {
        Err(StorageError::backend(format!(
            "legacy SQLite integrity check failed: {result}"
        )))
    }
}

fn inventory(conn: &Connection) -> Result<BTreeMap<&'static str, i64>, StorageError> {
    let mut counts = BTreeMap::new();
    for &table in OLD_TABLES {
        if !table_exists(conn, table).map_err(|e| error("inspect tables", e))? {
            return Err(StorageError::invalid_input(format!(
                "legacy SQLite storage is missing {table}; cannot upgrade safely"
            )));
        }
        let count: i64 = conn
            .query_row(&format!("SELECT count(*) FROM \"{table}\""), [], |row| {
                row.get(0)
            })
            .map_err(|e| error("inventory", e))?;
        counts.insert(table, count);
    }
    Ok(counts)
}

fn backup(
    conn: &Connection,
    file: &Path,
    before: &BTreeMap<&str, i64>,
) -> Result<PathBuf, StorageError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    let name = format!(
        "{}.pre-v2-{}-{stamp}.bak",
        file.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    );
    let path = file.with_file_name(name);
    if path.exists() {
        return Err(StorageError::backend(
            "SQLite backup already exists; migration stopped",
        ));
    }
    conn.execute("VACUUM INTO ?1", [path.to_string_lossy().as_ref()])
        .map_err(|e| error("create consistent pre-upgrade backup", e))?;
    let copy = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| error("open backup", e))?;
    check_integrity(&copy)?;
    if &inventory(&copy)? != before {
        return Err(StorageError::backend(
            "legacy SQLite changed during backup; migration stopped",
        ));
    }
    // On Windows, File::open obtains a read-only HANDLE, but sync_all uses
    // FlushFileBuffers, which requires GENERIC_WRITE. Close the read-only
    // SQLite verification connection, then open the already-verified backup
    // with read/write access solely for a durable flush. No bytes are changed.
    drop(copy);
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .and_then(|file| file.sync_all())
        .map_err(|e| error("flush backup", e))?;
    Ok(path)
}

fn legacy_time(doc: &Value, fields: &[&str]) -> Option<i64> {
    fields.iter().find_map(|key| {
        let value = doc.get(*key)?;
        match value {
            Value::String(text) => chrono::DateTime::parse_from_rfc3339(text)
                .ok()
                .map(|time| time.timestamp_millis()),
            Value::Number(number) => {
                let seconds = !key.ends_with("_ms");
                if let Some(number) = number.as_i64() {
                    if seconds && number.unsigned_abs() < 100_000_000_000 {
                        number.checked_mul(1000)
                    } else {
                        Some(number)
                    }
                } else {
                    let number = number.as_f64()?;
                    if !number.is_finite() {
                        return None;
                    }
                    let milliseconds = if seconds && number.abs() < 1e11 {
                        number * 1000.0
                    } else {
                        number
                    };
                    #[allow(clippy::cast_possible_truncation)]
                    (milliseconds.abs() < 9e18).then(|| milliseconds.round() as i64)
                }
            }
            _ => None,
        }
    })
}

fn value_kind(value: &Value) -> i64 {
    match value {
        Value::Null => 0,
        Value::Bool(_) => 1,
        Value::Number(_) => 2,
        Value::String(_) => 3,
        Value::Array(_) => 4,
        Value::Object(_) => 5,
    }
}

fn scalar(value: &Value) -> SqlValue {
    match value {
        Value::Null | Value::Array(_) | Value::Object(_) => SqlValue::Null,
        Value::Bool(flag) => SqlValue::Integer(i64::from(*flag)),
        Value::Number(number) => number.as_i64().map_or_else(
            || number.as_f64().map_or(SqlValue::Null, SqlValue::Real),
            SqlValue::Integer,
        ),
        Value::String(text) => SqlValue::Text(text.clone()),
    }
}

// Keep these DDL declarations in sync with the pinned TSD v2 SqliteTables.
// Validation on the subsequent normal driver open catches any schema drift.
fn create_layout_two(conn: &Connection) -> Result<(), StorageError> {
    conn.execute_batch(
        r#"CREATE TABLE IF NOT EXISTS "_tsd_scope_admissions" (
              scope TEXT PRIMARY KEY, generation INTEGER NOT NULL,
              operation TEXT NOT NULL) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "_tsd_purge_receipts" (
              scope TEXT NOT NULL, operation TEXT NOT NULL,
              generation INTEGER NOT NULL,
              PRIMARY KEY (scope, operation)) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "docs" (
              scope TEXT NOT NULL, coll TEXT NOT NULL, id TEXT NOT NULL,
              version INTEGER NOT NULL, schema INTEGER NOT NULL DEFAULT 0,
              created_ms INTEGER NOT NULL, updated_ms INTEGER NOT NULL,
              expires_ms INTEGER, deleted INTEGER NOT NULL DEFAULT 0,
              body TEXT NOT NULL, PRIMARY KEY(scope, coll, id)) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "doc_fields" (
              scope TEXT NOT NULL, coll TEXT NOT NULL, id TEXT NOT NULL,
              path TEXT NOT NULL, kind INTEGER NOT NULL, value,
              PRIMARY KEY(scope, coll, id, path)) WITHOUT ROWID;
           CREATE VIRTUAL TABLE IF NOT EXISTS "fts" USING fts5(
              scope UNINDEXED, coll UNINDEXED, id UNINDEXED, body,
              tokenize = 'unicode61');
           CREATE TABLE IF NOT EXISTS "streams" (
              scope TEXT NOT NULL, name TEXT NOT NULL, seq INTEGER NOT NULL,
              body TEXT NOT NULL, PRIMARY KEY(scope, name, seq)) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "stream_heads" (
              scope TEXT NOT NULL, name TEXT NOT NULL, base INTEGER NOT NULL,
              len INTEGER NOT NULL, sealed INTEGER NOT NULL DEFAULT 0,
              sealed_ms INTEGER, last_append_ms INTEGER,
              PRIMARY KEY(scope, name)) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "blobs" (
              scope TEXT NOT NULL, key TEXT NOT NULL, content_type TEXT,
              size INTEGER NOT NULL, sha256 BLOB NOT NULL, expires_ms INTEGER,
              bytes BLOB NOT NULL, PRIMARY KEY(scope, key)) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "meta" (
              key TEXT PRIMARY KEY, value TEXT NOT NULL) WITHOUT ROWID;
           CREATE TABLE IF NOT EXISTS "_tsd_blob_written" (
              scope TEXT NOT NULL, key TEXT NOT NULL, written_ms INTEGER NOT NULL,
              PRIMARY KEY(scope, key)) WITHOUT ROWID;
           CREATE INDEX IF NOT EXISTS "blobs_expiry"
              ON "blobs" (expires_ms) WHERE expires_ms IS NOT NULL;
           CREATE INDEX IF NOT EXISTS "doc_fields_lookup"
              ON "doc_fields" (scope, coll, path, value, id);"#,
    )
    .map_err(|e| error("create new tables", e))
}

fn specs(conn: &Connection) -> Result<BTreeMap<String, CollectionSpec>, StorageError> {
    let mut stmt = conn
        .prepare("SELECT spec FROM _tsd_collections")
        .map_err(|e| error("read collection metadata", e))?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(|e| error("read collection metadata", e))?;
    let mut result = BTreeMap::new();
    for row in rows {
        let text = row.map_err(|e| error("read collection metadata", e))?;
        let spec: CollectionSpec = serde_json::from_str(&text)
            .map_err(|e| error("parse historical collection spec", e))?;
        spec.validate()?;
        result.insert(spec.name.clone(), spec);
    }
    Ok(result)
}

fn copy_documents(conn: &Connection, now_ms: i64) -> Result<(), StorageError> {
    let specs = specs(conn)?;
    let mut stmt = conn
        .prepare("SELECT scope,coll,id,version,doc FROM _tsd_docs")
        .map_err(|e| error("read legacy documents", e))?;
    let mut rows = stmt
        .query([])
        .map_err(|e| error("read legacy documents", e))?;
    while let Some(row) = rows.next().map_err(|e| error("read legacy document", e))? {
        let scope: String = row.get(0).map_err(|e| error("document scope", e))?;
        let coll: String = row.get(1).map_err(|e| error("document collection", e))?;
        let id: String = row.get(2).map_err(|e| error("document id", e))?;
        let version: i64 = row.get(3).map_err(|e| error("document version", e))?;
        let body: String = row.get(4).map_err(|e| error("document body", e))?;
        let doc: Value = serde_json::from_str(&body)
            .map_err(|e| error("invalid historical document JSON", e))?;
        let created =
            legacy_time(&doc, &["created_ms", "created_at", "createdAt"]).unwrap_or(now_ms);
        let updated = legacy_time(&doc, &["updated_ms", "updated_at", "updatedAt"])
            .unwrap_or(created)
            .max(created);
        let expires = specs
            .get(&coll)
            .and_then(|spec| spec.ttl_field.as_deref())
            .and_then(|path| value::lookup(&doc, path))
            .and_then(Value::as_i64);
        conn.execute(
            "INSERT INTO docs (scope,coll,id,version,schema,created_ms,updated_ms,expires_ms,deleted,body)
             VALUES (?1,?2,?3,?4,0,?5,?6,?7,0,?8)",
            params![scope, coll, id, version, created, updated, expires, body],
        )
        .map_err(|e| error("copy document", e))?;
        if let Some(spec) = specs.get(&coll) {
            let paths: BTreeSet<&str> = spec
                .indexes
                .iter()
                .flat_map(|index| index.fields.iter().map(String::as_str))
                .chain(
                    spec.search
                        .iter()
                        .flat_map(|search| search.fields.iter().map(String::as_str)),
                )
                .collect();
            for path in paths {
                if let Some(found) = value::lookup(&doc, path) {
                    conn.execute(
                        "INSERT INTO doc_fields (scope,coll,id,path,kind,value)
                         VALUES (?1,?2,?3,?4,?5,?6)",
                        params![scope, coll, id, path, value_kind(found), scalar(found)],
                    )
                    .map_err(|e| error("project indexed document fields", e))?;
                }
            }
        }
    }
    Ok(())
}

fn copy_blobs(conn: &Connection) -> Result<(), StorageError> {
    let expiry = if has_column(conn, "_tsd_blobs", "expires_at_ms")
        .map_err(|e| error("inspect historical blob expiry", e))?
    {
        "expires_at_ms"
    } else {
        "NULL"
    };
    let mut stmt = conn
        .prepare(&format!(
            "SELECT scope,key,content_type,bytes,{expiry} FROM _tsd_blobs"
        ))
        .map_err(|e| error("read legacy blobs", e))?;
    let mut rows = stmt.query([]).map_err(|e| error("read legacy blobs", e))?;
    while let Some(row) = rows.next().map_err(|e| error("read legacy blob", e))? {
        let scope: String = row.get(0).map_err(|e| error("blob scope", e))?;
        let key: String = row.get(1).map_err(|e| error("blob key", e))?;
        let content_type: Option<String> = row.get(2).map_err(|e| error("blob media type", e))?;
        let bytes: Vec<u8> = row.get(3).map_err(|e| error("blob bytes", e))?;
        let expires: Option<i64> = row.get(4).map_err(|e| error("blob expiry", e))?;
        let size = i64::try_from(bytes.len()).map_err(|e| error("blob size", e))?;
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        conn.execute(
            "INSERT INTO blobs (scope,key,content_type,size,sha256,expires_ms,bytes)
             VALUES (?1,?2,?3,?4,?5,?6,?7)",
            params![
                scope,
                key,
                content_type,
                size,
                digest.to_vec(),
                expires,
                bytes
            ],
        )
        .map_err(|e| error("copy blob", e))?;
    }
    Ok(())
}

fn count(conn: &Connection, table: &str) -> Result<i64, StorageError> {
    conn.query_row(&format!("SELECT count(*) FROM \"{table}\""), [], |row| {
        row.get(0)
    })
    .map_err(|e| error("verify migrated table", e))
}

fn convert(conn: &Connection, before: &BTreeMap<&str, i64>) -> Result<(), StorageError> {
    create_layout_two(conn)?;
    copy_documents(conn, chrono::Utc::now().timestamp_millis())?;
    conn.execute_batch(
        "INSERT OR IGNORE INTO docs
           (scope,coll,id,version,schema,created_ms,updated_ms,expires_ms,deleted,body)
         SELECT scope,coll,id,version,0,0,0,NULL,1,'{}' FROM _tsd_tombstones;
         INSERT INTO fts (scope,coll,id,body)
           SELECT scope,coll,id,body FROM _tsd_fts;
         INSERT INTO streams (scope,name,seq,body)
           SELECT scope,stream,seq,value FROM _tsd_streams;",
    )
    .map_err(|e| error("copy tombstones, search and streams", e))?;
    let sealed = if has_column(conn, "_tsd_stream_meta", "sealed")
        .map_err(|e| error("inspect stream seal", e))?
    {
        "sealed"
    } else {
        "0"
    };
    conn.execute_batch(&format!(
        "INSERT INTO stream_heads (scope,name,base,len,sealed)
         SELECT scope,stream,base,len,{sealed} FROM _tsd_stream_meta"
    ))
    .map_err(|e| error("copy stream heads", e))?;
    copy_blobs(conn)?;
    let distinct_documents: i64 = conn
        .query_row(
            "SELECT count(*) FROM (
               SELECT scope,coll,id FROM _tsd_docs
               UNION SELECT scope,coll,id FROM _tsd_tombstones
             )",
            [],
            |row| row.get(0),
        )
        .map_err(|e| error("verify historical documents", e))?;
    for (target, expected) in [
        ("docs", distinct_documents),
        ("fts", before["_tsd_fts"]),
        ("streams", before["_tsd_streams"]),
        ("stream_heads", before["_tsd_stream_meta"]),
        ("blobs", before["_tsd_blobs"]),
    ] {
        if count(conn, target)? != expected {
            return Err(StorageError::backend(format!(
                "legacy SQLite migration count mismatch in {target}; source untouched"
            )));
        }
    }
    // The v2 driver compares persisted CREATE TABLE statements, including
    // quoted identifiers. A v0 _tsd_collections table may have been created
    // without quotes: keeping its original DDL causes every v2 open to fail
    // even when all other migrated tables are valid. Rebuild it transactionally,
    // preserving all collection specs and the driver's canonical DDL.
    conn.execute_batch(
        "CREATE TABLE \"_tsd_collections_next\" (
             coll TEXT PRIMARY KEY, spec TEXT NOT NULL
         ) WITHOUT ROWID;
         INSERT INTO \"_tsd_collections_next\" (coll,spec)
           SELECT coll,spec FROM \"_tsd_collections\";
         DROP TABLE \"_tsd_collections\";
         ALTER TABLE \"_tsd_collections_next\" RENAME TO \"_tsd_collections\";",
    )
    .map_err(|e| error("rebuild legacy collection schema", e))?;
    if count(conn, "_tsd_collections")? != before["_tsd_collections"] {
        return Err(StorageError::backend(
            "legacy SQLite collection specs changed during migration",
        ));
    }
    for table in OLD_TABLES {
        if *table != "_tsd_collections" {
            conn.execute_batch(&format!("DROP TABLE \"{table}\""))
                .map_err(|e| error("retire copied legacy table", e))?;
        }
    }
    conn.execute(
        "INSERT OR REPLACE INTO meta (key,value) VALUES ('layout','2')",
        [],
    )
    .map_err(|e| error("record new layout", e))?;
    check_integrity(conn)
}

/// Upgrade a historical SQLite database, or return without touching an
/// absent/current database. Keeps an independently verified rollback copy.
/// Failure leaves the original database in its previous format.
pub(crate) fn upgrade_if_needed(path: &Path) -> Result<(), StorageError> {
    let file = database_path(path);
    if !file.is_file() {
        return Ok(());
    }
    // The overwhelmingly common v2 path is read-only and creates no sidecar
    // file or lock. Only historical _tsd_docs databases need coordination.
    let probe = Connection::open_with_flags(&file, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| error("inspect existing database", e))?;
    if !table_exists(&probe, "_tsd_docs").map_err(|e| error("inspect legacy layout", e))? {
        return Ok(());
    }
    drop(probe);
    // A second OpenHuman process must re-check the current schema *after*
    // acquiring the lock: the first process may already have committed v2.
    let _migration_lock = MigrationLock::acquire(&file)?;
    let conn = Connection::open_with_flags(&file, OpenFlags::SQLITE_OPEN_READ_WRITE)
        .map_err(|e| error("open existing database", e))?;
    if !table_exists(&conn, "_tsd_docs").map_err(|e| error("inspect legacy layout", e))? {
        return Ok(());
    }
    if table_exists(&conn, "docs").map_err(|e| error("inspect existing layout", e))?
        || table_exists(&conn, "meta").map_err(|e| error("inspect existing layout", e))?
    {
        return Err(StorageError::invalid_input(
            "SQLite contains mixed legacy and v2 tables; refusing an ambiguous upgrade",
        ));
    }
    check_integrity(&conn)?;
    let before = inventory(&conn)?;
    preflight_migration_disk_space(&file)?;
    // A peer can update a record without changing row counts. SQLite
    // data_version catches that write between the consistent snapshot and
    // our exclusive migration transaction.
    let version_before: i64 = conn
        .query_row("PRAGMA data_version", [], |row| row.get(0))
        .map_err(|e| error("read source version", e))?;
    let rollback_copy = backup(&conn, &file, &before)?;
    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|e| error("lock database for upgrade", e))?;
    let result = (|| {
        if !table_exists(&conn, "_tsd_docs").map_err(|e| error("reinspect layout", e))? {
            return Err(StorageError::backend(
                "legacy SQLite changed while preparing upgrade; retry startup",
            ));
        }
        let version_now: i64 = conn
            .query_row("PRAGMA data_version", [], |row| row.get(0))
            .map_err(|e| error("recheck source version", e))?;
        if version_now != version_before || inventory(&conn)? != before {
            return Err(StorageError::backend(
                "legacy SQLite changed after backup; retry startup",
            ));
        }
        convert(&conn, &before)
    })();
    if let Err(error) = result {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(error);
    }
    if let Err(commit_error) = conn.execute_batch("COMMIT") {
        let _ = conn.execute_batch("ROLLBACK");
        return Err(error("commit upgrade", commit_error));
    }
    tracing::info!(
        target: "openhuman::storage",
        database = %file.display(),
        rollback_copy = %rollback_copy.display(),
        "legacy SQLite layout converted to v2 with verified backup"
    );
    Ok(())
}

#[cfg(all(test, feature = "storage-sqlite"))]
#[path = "sqlite_v0_tests.rs"]
mod tests;
