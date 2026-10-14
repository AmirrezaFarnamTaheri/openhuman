//! Conservative, derived file-dependency projection.
//!
//! Inspired by Graphify / CodeGraph, but not a new persistent graph store.
//! Edges express observed syntax and exact catalog matching; absent, duplicated
//! or unsupported resolutions NEVER become confirmed dependencies.
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;
use serde_json::{json, Value};

/// Original text is only retained for the duration of a bounded graph call.
pub(super) struct SourceDocument {
    pub path: String,
    pub source_hash: String,
    pub content: String,
}

fn pattern(cache: &'static OnceLock<Regex>, expression: &'static str) -> &'static Regex {
    cache.get_or_init(|| Regex::new(expression).expect("literal regex is valid"))
}
fn rust_mod_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    pattern(&PATTERN, r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z_0-9]*)\s*;")
}
fn js_from_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    pattern(&PATTERN, r#"^\s*(?:import|export)\b.*?\bfrom\s*['"](\.[^'"]+)['"]"#)
}
fn js_side_effect_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    pattern(&PATTERN, r#"^\s*import\s*['"](\.[^'"]+)['"]"#)
}
fn js_require_pattern() -> &'static Regex {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    pattern(&PATTERN, r#"\brequire\s*\(\s*['"](\.[^'"]+)['"]\s*\)"#)
}

/// Normalize an import specifier, not a filesystem path; this performs no I/O.
fn relative_target(source: &str, spec: &str) -> Option<String> {
    if !spec.starts_with('.') || spec.contains('\0') || spec.contains('\\') {
        return None;
    }
    let mut parts: Vec<&str> = source.split('/').collect();
    parts.pop();
    for segment in spec.split('/') {
        match segment {
            "" | "." => {}
            ".." => { parts.pop()?; }
            part => parts.push(part),
        }
    }
    if parts.is_empty() { return None; }
    Some(parts.join("/"))
}

fn candidate_paths(source: &str, spec: &str, kind: &str) -> Vec<String> {
    if kind == "module" {
        // foo.rs nested-module directory convention differs from lib/main/mod.
        let mut parent = source.rsplit_once('/').map_or("", |(dir, _)| dir).to_string();
        let name = Path::new(source).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !matches!(name, "lib" | "main" | "mod") {
            if !parent.is_empty() { parent.push('/'); }
            parent.push_str(name);
        }
        let prefix = if parent.is_empty() { String::new() } else { format!("{parent}/") };
        return vec![format!("{prefix}{spec}.rs"), format!("{prefix}{spec}/mod.rs")];
    }
    let Some(stem) = relative_target(source, spec) else { return Vec::new(); };
    if matches!(
        Path::new(&stem).extension().and_then(|ext| ext.to_str()),
        Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts")
    ) {
        vec![stem]
    } else {
        ["ts", "tsx", "js", "jsx", "mjs", "cjs", "mts", "cts"]
            .iter()
            .map(|ext| format!("{stem}.{ext}"))
            .chain(["ts", "tsx", "js", "jsx"].iter().map(|ext| format!("{stem}/index.{ext}")))
            .collect()
    }
}

fn resolve(present: &BTreeMap<&str, &str>, candidates: Vec<String>) -> (&'static str, Option<String>, usize) {
    let matches: Vec<String> = candidates.into_iter()
        .filter(|name| present.contains_key(name.as_str()))
        .collect();
    match matches.len() {
        0 => ("unresolved", None, 0),
        1 => ("resolved", matches.into_iter().next(), 1),
        n => ("ambiguous", None, n),
    }
}

/// A purely derived graph; no filesystem access, LLM call, or persistence.
/// Impact is a bounded lower bound based only on confirmed observed edges.
pub(super) fn build_graph(docs: &[SourceDocument], max_edges: usize, focus: Option<&str>) -> Value {
    let mut ordered: Vec<&SourceDocument> = docs.iter().collect();
    ordered.sort_by(|a, b| a.path.cmp(&b.path));
    let present: BTreeMap<&str, &str> = ordered.iter()
        .map(|doc| (doc.path.as_str(), doc.source_hash.as_str())).collect();
    let nodes: Vec<Value> = ordered.iter()
        .map(|doc| json!({"path":doc.path,"source_hash":doc.source_hash})).collect();

    let mut edges = Vec::<Value>::new();
    let mut reverse = BTreeMap::<String, BTreeSet<String>>::new();
    let mut complete = true;
    let max_edges = max_edges.min(64);
    'documents: for doc in ordered {
        let ext = Path::new(&doc.path).extension().and_then(|e| e.to_str());
        let js = matches!(ext, Some("ts" | "tsx" | "js" | "jsx" | "mjs" | "cjs" | "mts" | "cts"));
        let rust = ext == Some("rs");
        if !js && !rust { continue; }
        let mut previous_line = "";
        for (index, line) in doc.content.lines().enumerate() {
            let import = if rust {
                if previous_line.trim_start().starts_with("#[path") {
                    previous_line = line;
                    continue;
                }
                rust_mod_pattern().captures(line)
                    .map(|captures| ("module", captures[1].to_string()))
            } else {
                js_from_pattern().captures(line)
                    .or_else(|| js_side_effect_pattern().captures(line))
                    .or_else(|| js_require_pattern().captures(line))
                    .map(|captures| ("import", captures[1].to_string()))
            };
            previous_line = line;
            let Some((kind, spec)) = import else { continue; };
            let (status, target, count) = resolve(&present, candidate_paths(&doc.path, &spec, kind));
            if edges.len() >= max_edges {
                complete = false;
                break 'documents;
            }
            if let Some(to) = target.as_ref() {
                reverse.entry(to.clone()).or_default().insert(doc.path.clone());
            }
            edges.push(json!({
                "from":doc.path,
                "to":target,
                "kind":kind,
                "status":status,
                "candidate_count":count,
                "line":index+1,
                "specifier":spec.chars().take(120).collect::<String>(),
                "evidence_hash":doc.source_hash,
                "confidence":"syntactic_only"
            }));
        }
    }

    let indexed_focus = focus.is_some_and(|path| present.contains_key(path));
    let mut impacted = BTreeSet::new();
    if let Some(focus_path) = focus.filter(|_| indexed_focus) {
        let mut pending = VecDeque::from([focus_path.to_owned()]);
        while let Some(target) = pending.pop_front() {
            if let Some(sources) = reverse.get(&target) {
                for parent in sources {
                    if parent != focus_path && impacted.insert(parent.clone()) {
                        pending.push_back(parent.clone());
                    }
                }
            }
        }
    }

    let mut graph = json!({
        "operation":"graph",
        "nodes":nodes,
        "edges":edges,
        "edges_complete":complete,
        "impact":impacted.into_iter().collect::<Vec<_>>(),
        "focus_indexed":indexed_focus,
        "impact_exhaustive":false,
        "semantic_graph":false,
        "snapshot_atomic":false,
        "supported_syntax":["rust_mod", "javascript_typescript_static_relative_import", "commonjs_relative_require"],
        "resolution_rule":"exact unique local candidate only"
    });
    // Tool output may be threaded directly into an LLM context. Preserve a
    // parseable JSON envelope instead of relying on downstream character
    // truncation, which would corrupt it and drop completeness metadata.
    const MAX_GRAPH_JSON_BYTES: usize = 26_000;
    while graph.to_string().len() > MAX_GRAPH_JSON_BYTES {
        let Some(edges) = graph["edges"].as_array_mut() else { break; };
        if edges.pop().is_none() {
            break;
        }
        graph["edges_complete"] = json!(false);
        graph["edge_truncated_reason"] = json!("max_graph_result_bytes");
        // Do not leave dependency impact claims that rely on relationships
        // deliberately omitted from the bounded response.
        graph["impact"] = json!([]);
        graph["impact_omitted_reason"] = json!("bounded_edges");
    }
    graph
}

#[cfg(test)]
#[path = "source_graph_tests.rs"]
mod tests;
