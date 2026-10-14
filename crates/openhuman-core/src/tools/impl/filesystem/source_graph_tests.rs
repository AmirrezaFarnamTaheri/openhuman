//! Regression contract for a bounded, derived file-dependency graph.
use super::{build_graph, SourceDocument};
use serde_json::Value;
use sha2::Digest;

fn source(path: &str, content: &str) -> SourceDocument {
    SourceDocument {
        path: path.to_owned(),
        source_hash: format!("{:x}", sha2::Sha256::digest(content.as_bytes())),
        content: content.to_owned(),
    }
}

fn edge<'a>(graph: &'a Value) -> &'a Value {
    &graph["edges"].as_array().expect("edges array")[0]
}

#[test]
fn javascript_relative_imports_are_resolved_only_from_observed_files() {
    let docs = vec![
        source("src/main.ts", "import { load } from './utils';\n"),
        source("src/utils.ts", "export function load() {}\n"),
    ];
    let graph = build_graph(&docs, 64, None);
    assert_eq!(graph["nodes"].as_array().unwrap().len(), 2);
    assert_eq!(edge(&graph)["kind"], "import");
    assert_eq!(edge(&graph)["status"], "resolved");
    assert_eq!(edge(&graph)["from"], "src/main.ts");
    assert_eq!(edge(&graph)["to"], "src/utils.ts");
    assert_eq!(edge(&graph)["line"], 1);
    assert_eq!(edge(&graph)["evidence_hash"], docs[0].source_hash);
}

#[test]
fn ambiguous_relative_import_is_not_selected_arbitrarily() {
    let docs = vec![
        source("src/main.ts", "import './helpers';\n"),
        source("src/helpers.ts", "export const a = 1;\n"),
        source("src/helpers.js", "exports.a = 1;\n"),
    ];
    let graph = build_graph(&docs, 64, None);
    assert_eq!(edge(&graph)["status"], "ambiguous");
    assert!(edge(&graph)["to"].is_null());
    assert_eq!(edge(&graph)["candidate_count"], 2);
}

#[test]
fn missing_and_out_of_root_references_are_unresolved() {
    let docs = vec![source(
        "src/index.js",
        "import other from './missing.js';\nimport outside from '../../private';\n",
    )];
    let graph = build_graph(&docs, 64, None);
    assert_eq!(graph["edges"].as_array().unwrap().len(), 2);
    for relation in graph["edges"].as_array().unwrap() {
        assert_eq!(relation["status"], "unresolved");
        assert!(relation["to"].is_null());
    }
    assert!(!graph.to_string().contains("private/secret"));
}

#[test]
fn rust_module_paths_resolve_to_their_actual_file() {
    let docs = vec![
        source("src/lib.rs", "pub mod api;\n"),
        source("src/api/mod.rs", "pub fn list() {}\n"),
    ];
    let graph = build_graph(&docs, 64, None);
    assert_eq!(edge(&graph)["kind"], "module");
    assert_eq!(edge(&graph)["status"], "resolved");
    assert_eq!(edge(&graph)["to"], "src/api/mod.rs");
}

#[test]
fn ambiguous_rust_module_paths_are_never_assumed() {
    let docs = vec![
        source("src/lib.rs", "mod api;\n"),
        source("src/api.rs", "pub fn one() {}\n"),
        source("src/api/mod.rs", "pub fn two() {}\n"),
    ];
    let graph = build_graph(&docs, 64, None);
    assert_eq!(edge(&graph)["status"], "ambiguous");
}

#[test]
fn impact_follows_only_confirmed_inbound_dependencies() {
    let docs = vec![
        source("src/a.ts", "import './b';\n"),
        source("src/b.ts", "export const value = 1;\n"),
        source("src/c.ts", "import './a';\n"),
        source("src/d.ts", "import './missing';\n"),
    ];
    let graph = build_graph(&docs, 64, Some("src/b.ts"));
    assert_eq!(graph["impact"], serde_json::json!(["src/a.ts", "src/c.ts"]));
    assert_eq!(graph["impact_exhaustive"], false);
}

#[test]
fn edge_budget_reports_partial_projection() {
    let docs = vec![
        source("src/a.ts", "import './b';\nimport './c';\n"),
        source("src/b.ts", "export const b = 1;\n"),
        source("src/c.ts", "export const c = 1;\n"),
    ];
    let graph = build_graph(&docs, 1, None);
    assert_eq!(graph["edges"].as_array().unwrap().len(), 1);
    assert_eq!(graph["edges_complete"], false);
}

#[test]
fn no_claim_of_complete_language_or_semantic_coverage() {
    let graph = build_graph(&[source("app/main.py", "import requests\n")], 64, None);
    assert_eq!(graph["semantic_graph"], false);
    assert_eq!(graph["snapshot_atomic"], false);
    assert_eq!(graph["edges"].as_array().unwrap().len(), 0);
}
