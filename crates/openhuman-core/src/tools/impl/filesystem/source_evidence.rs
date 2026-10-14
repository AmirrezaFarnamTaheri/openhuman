//! Read-only, source-hash anchored workspace evidence.
//!
//! Adapted from the evidence/scope contracts of Jev and Conduit. This is not
//! a durable index: each call checks the host SecurityPolicy and observed bytes.
use std::path::{Path, PathBuf};
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use tinytools::{PermissionLevel, Tool, ToolCallOptions, ToolResult, ToolRunContext};

use crate::security::SecurityPolicy;
use super::gate::security_scoped_to_root;

const MAX_FILES: usize = 32;
const MAX_FILE_BYTES: u64 = 256 * 1024;
const MAX_TOTAL_BYTES: usize = 2 * 1024 * 1024;
const MAX_MATCHES: usize = 3;
const MAX_LINE_CHARS: usize = 160;
const MAX_DEPTH: usize = 12;
const MAX_VERIFY_LINES: usize = 80;
const SKIP_DIRS: &[&str] = &[".git", ".svn", ".hg", "node_modules", "target", ".next", "dist", "build", ".venv", "__pycache__"];
const SKIP_FILES: &[&str] = &[".env", ".env.local", ".env.production", ".env.development", ".npmrc", ".pypirc", "id_rsa", "id_ed25519", "credentials.json", "secrets.json"];

pub struct SourceEvidenceTool {
    security: Arc<SecurityPolicy>,
}
impl SourceEvidenceTool {
    pub fn new(security: Arc<SecurityPolicy>) -> Self { Self { security } }

    async fn run(&self, args: Value, context: Option<&dyn ToolRunContext>) -> anyhow::Result<ToolResult> {
        let scoped = match context.and_then(|ctx| ctx.workspace()) {
            Some(ws) => security_scoped_to_root(&self.security, &ws.root),
            None => (*self.security).clone(),
        };
        let op = args.get("operation").and_then(Value::as_str).unwrap_or("scan");
        let path = args.get("path").and_then(Value::as_str).unwrap_or(".");
        let resolved = scoped.validate_path(path).await.map_err(anyhow::Error::msg)?;
        let root = scoped.validate_path(".").await.map_err(anyhow::Error::msg)?;
        if !resolved.starts_with(&root) {
            anyhow::bail!("source evidence path is outside the active workspace");
        }
        let result = match op {
            "scan" => scan(&scoped, &root, &resolved, &args).await?,
            "verify" => verify(&scoped, &root, &resolved, &args).await?,
            _ => anyhow::bail!("operation must be scan or verify"),
        };
        Ok(ToolResult::success(result.to_string()))
    }
}

#[async_trait]
impl Tool for SourceEvidenceTool {
    fn name(&self) -> &str { "source_evidence" }
    fn description(&self) -> &str {
        "Opt-in, read-only source search and content-hash evidence in the active workspace.          Scan source files with a bounded optional query; verify a file digest before reading          line-numbered excerpts. No source is sent to a remote model by this tool."
    }
    fn permission_level(&self) -> PermissionLevel { PermissionLevel::ReadOnly }
    fn parameters_schema(&self) -> Value {
        json!({"type":"object","properties":{
            "operation":{"type":"string","enum":["scan","verify"]},
            "path":{"type":"string","description":"Relative path in current workspace, default '.'"},
            "query":{"type":"string","maxLength":128,"description":"Optional literal substring for scanning"},
            "max_files":{"type":"integer","minimum":1,"maximum":32},
            "expected_sha256":{"type":"string","minLength":64,"maxLength":64},
            "start_line":{"type":"integer","minimum":1},
            "end_line":{"type":"integer","minimum":1}
        }})
    }
    fn max_result_size_chars(&self) -> Option<usize> { Some(30_000) }
    fn is_concurrency_safe(&self, _args: &Value) -> bool { true }
    async fn execute(&self, args: Value) -> anyhow::Result<ToolResult> {
        self.run(args, None).await
    }
    async fn execute_with_context(&self, args: Value, _options: ToolCallOptions, context: Option<&dyn ToolRunContext>) -> anyhow::Result<ToolResult> {
        self.run(args, context).await
    }
}

fn ignored(path: &Path, directory: bool) -> bool {
    let Some(name) = path.file_name().and_then(|x| x.to_str()) else { return true };
    if directory { return SKIP_DIRS.contains(&name) || name.starts_with(".cache"); }
    let lower = name.to_ascii_lowercase();
    SKIP_FILES.contains(&lower.as_str()) ||
        lower.starts_with(".env.") ||
        lower.ends_with(".pem") || lower.ends_with(".key") ||
        lower.ends_with(".p12") || lower.ends_with(".pfx") ||
        lower.ends_with(".sqlite") || lower.ends_with(".db")
}
fn hex_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
fn bounded_line(s: &str) -> String {
    s.chars().take(MAX_LINE_CHARS).collect()
}
fn parse_line_range(args: &Value) -> anyhow::Result<(usize, usize)> {
    let start = args.get("start_line").and_then(Value::as_u64).unwrap_or(1);
    let end = args.get("end_line").and_then(Value::as_u64).unwrap_or(start.saturating_add(39));
    if start == 0 || end < start || end - start >= MAX_VERIFY_LINES as u64 {
        anyhow::bail!("invalid line range (at most {MAX_VERIFY_LINES} lines)");
    }
    Ok((usize::try_from(start)?, usize::try_from(end)?))
}
async fn read_checked(policy: &SecurityPolicy, path: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    // Revalidate each individual file: cataloging a directory does not grant
    // read permissions to its denied descendants.
    let actual = policy.validate_path(&path.to_string_lossy()).await;
    let Ok(actual) = actual else { return Ok(None); };
    let metadata = tokio::fs::symlink_metadata(path).await?;
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > MAX_FILE_BYTES {
        return Ok(None);
    }
    let bytes = tokio::fs::read(&actual).await?;
    if bytes.len() > MAX_FILE_BYTES as usize || bytes.contains(&0) || std::str::from_utf8(&bytes).is_err() {
        return Ok(None);
    }
    Ok(Some(bytes))
}
async fn scan(policy: &SecurityPolicy, root: &Path, path: &Path, args: &Value) -> anyhow::Result<Value> {
    let query = args.get("query").and_then(Value::as_str).unwrap_or("");
    if query.len() > 128 { anyhow::bail!("query too long"); }
    let max_files = args.get("max_files").and_then(Value::as_u64).unwrap_or(MAX_FILES as u64).clamp(1, MAX_FILES as u64) as usize;
    let mut paths = vec![(path.to_path_buf(), 0usize)];
    let mut entries: Vec<PathBuf> = Vec::new();
    let mut truncated: Option<&str> = None;
    // Bounded traversal, deterministic ordering; never follow directory links.
    while let Some((next, depth)) = paths.pop() {
        let metadata = match tokio::fs::symlink_metadata(&next).await {
            Ok(meta) => meta,
            Err(_) => { truncated = Some("filesystem_changed"); continue; }
        };
        if metadata.file_type().is_symlink() { continue; }
        if metadata.is_file() {
            if !ignored(&next, false) { entries.push(next); }
            if entries.len() > max_files {
                truncated = Some("max_files");
                break;
            }
        } else if metadata.is_dir() && !ignored(&next, true) {
            if depth >= MAX_DEPTH { truncated = Some("max_depth"); continue; }
            if policy.validate_path(&next.to_string_lossy()).await.is_err() { continue; }
            let mut read = tokio::fs::read_dir(&next).await?;
            let mut children = Vec::new();
            while let Some(child) = read.next_entry().await? {
                children.push(child.path());
                if children.len() > 4096 { truncated = Some("directory_entry_limit"); break; }
            }
            children.sort();
            for child in children.into_iter().rev() { paths.push((child, depth+1)); }
        }
    }
    entries.sort();
    let mut files = Vec::new();
    let mut total = 0usize;
    let mut generation = Sha256::new();
    let mut source_bytes = 0usize;
    for path in entries.into_iter().take(max_files) {
        let metadata = tokio::fs::symlink_metadata(&path).await?;
        if metadata.len() > MAX_FILE_BYTES {
            truncated.get_or_insert("oversize_file");
            continue;
        }
        let Some(bytes) = read_checked(policy, &path).await? else { continue };
        if total.saturating_add(bytes.len()) > MAX_TOTAL_BYTES {
            truncated = Some("max_total_bytes"); break;
        }
        total += bytes.len();
        let digest = hex_digest(&bytes);
        let label = relative(root, &path);
        generation.update(label.as_bytes()); generation.update([0]); generation.update(digest.as_bytes()); generation.update([0]);
        let text = std::str::from_utf8(&bytes)?;
        let matches: Vec<Value> = if query.is_empty() { Vec::new() } else {
            let all: Vec<_> = text.lines().enumerate()
                .filter(|(_,line)| line.contains(query)).collect();
            if all.len() > MAX_MATCHES { truncated.get_or_insert("max_matches_per_file"); }
            all.into_iter().take(MAX_MATCHES).map(|(i,line)| json!({"line":i+1,"text":bounded_line(line)})).collect()
        };
        source_bytes += bytes.len();
        files.push(json!({"path":label,"source_hash":digest,"size_bytes":bytes.len(),"matches":matches}));
    }
    Ok(json!({
        "operation":"scan","generation":format!("{:x}",generation.finalize()),
        "complete":truncated.is_none(),"truncated_reason":truncated,
        "observed_bytes":source_bytes, "files":files,
        "snapshot_atomic":false
    }))
}
async fn verify(policy: &SecurityPolicy, root: &Path, path: &Path, args: &Value) -> anyhow::Result<Value> {
    let expected = args.get("expected_sha256").and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("expected_sha256 is required"))?;
    if expected.len() != 64 || !expected.bytes().all(|b| b.is_ascii_hexdigit()) {
        anyhow::bail!("expected_sha256 must be a 64-digit hexadecimal SHA-256");
    }
    let (start, end) = parse_line_range(args)?;
    if ignored(path, false) { anyhow::bail!("source file is excluded"); }
    let bytes = read_checked(policy, path).await?
        .ok_or_else(|| anyhow::anyhow!("source file is not permitted, not UTF-8, or too large"))?;
    let actual = hex_digest(&bytes);
    if !actual.eq_ignore_ascii_case(expected) { anyhow::bail!("source changed since evidence was recorded"); }
    let text = std::str::from_utf8(&bytes)?;
    let lines: Vec<Value> = text.lines().enumerate()
        .filter(|(i,_)| i+1 >= start && i+1 <= end)
        .map(|(i,line)| json!({"line":i+1,"text":bounded_line(line)})).collect();
    Ok(json!({"operation":"verify","path":relative(root,path),"source_hash":actual,
        "verified":true,"lines":lines,"snapshot_atomic":false}))
}

#[cfg(test)]
#[path = "source_evidence_tests.rs"]
mod tests;
