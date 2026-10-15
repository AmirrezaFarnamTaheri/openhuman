//! Explainable, deterministic, no-egress code location over authorized bytes.
//!
//! Inspired by jevex/blink search packets without introducing another index,
//! credential, model call, or store. It only ranks observed (hash-anchored)
//! source files; it NEVER asserts that an unobserved file is irrelevant.
use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::source_graph::SourceDocument;

const MAX_RESULTS: usize = 5;
const MAX_EVIDENCE_LINES: usize = 3;
const MAX_LINE_CHARS: usize = 160;
const MAX_JSON_BYTES: usize = 26_000;

const STOPWORDS: &[&str] = &[
    "a", "an", "and", "are", "as", "at", "be", "by", "can", "code",
    "do", "does", "file", "files", "find", "for", "from", "how", "i",
    "in", "is", "it", "me", "of", "on", "or", "please", "show", "the",
    "this", "to", "was", "were", "what", "when", "where", "which",
    "who", "why", "with", "would", "you",
];

fn tokenize(input: &str) -> BTreeSet<String> {
    let mut pieces = BTreeSet::new();
    let mut current = String::new();
    let mut last_lower = false;
    for ch in input.chars() {
        if !ch.is_alphanumeric() {
            if (2..=64).contains(&current.chars().count()) {
                pieces.insert(current.to_lowercase());
            }
            current.clear();
            last_lower = false;
            continue;
        }
        if last_lower && ch.is_uppercase() && !current.is_empty() {
            if (2..=64).contains(&current.chars().count()) {
                pieces.insert(current.to_lowercase());
            }
            current.clear();
        }
        if current.chars().count() < 64 {
            current.push(ch);
        }
        last_lower = ch.is_lowercase();
    }
    if (2..=64).contains(&current.chars().count()) {
        pieces.insert(current.to_lowercase());
    }
    pieces
}

fn query_terms(input: &str) -> BTreeSet<String> {
    tokenize(input)
        .into_iter()
        .filter(|token| !STOPWORDS.contains(&token.as_str()))
        .take(16)
        .collect()
}

fn evidence(doc: &SourceDocument, query: &BTreeSet<String>) -> (BTreeSet<String>, Vec<Value>) {
    let mut all_found = BTreeSet::new();
    let mut spans = Vec::new();
    for (line_number, line) in doc.content.lines().enumerate() {
        let tokens = tokenize(line);
        let hits: Vec<String> = query.intersection(&tokens).cloned().collect();
        if hits.is_empty() {
            continue;
        }
        all_found.extend(hits.iter().cloned());
        if spans.len() < MAX_EVIDENCE_LINES {
            spans.push(json!({
                "line": line_number + 1,
                "text": line.chars().take(MAX_LINE_CHARS).collect::<String>(),
                "matched_terms": hits
            }));
        }
    }
    (all_found, spans)
}

/// Produces bounded, source-hash-anchored *candidate* ranges, not verified
/// source-of-truth. Never performs I/O. Inputs must already have passed the
/// existing SourceEvidenceTool workspace/security/file-size policy.
pub(super) fn rank_sources(docs: &[SourceDocument], query: &str, limit: usize) -> Value {
    let terms = query_terms(query);
    let mut scores: Vec<(usize, String, Value)> = Vec::new();

    if !terms.is_empty() {
        for doc in docs {
            let path_tokens = tokenize(&doc.path);
            let path_hits: BTreeSet<String> = terms.intersection(&path_tokens).cloned().collect();
            let (body_hits, spans) = evidence(doc, &terms);
            let matched: BTreeSet<String> = path_hits.union(&body_hits).cloned().collect();
            if matched.is_empty() {
                continue;
            }
            // Explainable integer heuristic, not calibrated probability.
            // Path locality and distinct term coverage outrank repeated
            // occurrences in large generic documentation files.
            let score = path_hits.len() * 12 + body_hits.len() * 5 + matched.len() * 3;
            scores.push((
                score,
                doc.path.clone(),
                json!({
                    "path": doc.path,
                    "source_hash": doc.source_hash,
                    "score": score,
                    "matched_terms": matched,
                    "evidence": spans,
                    "evidence_verified": false
                }),
            ));
        }
    }
    scores.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    let total_candidates = scores.len();
    let mut candidates: Vec<Value> = scores.into_iter()
        .take(limit.clamp(1, MAX_RESULTS))
        .map(|(_, _, item)| item)
        .collect();

    // Host tool outputs are plain text and may otherwise be clipped into
    // syntactically-invalid JSON. Keep a coherent, explicitly partial result.
    let mut response = json!({
        "operation": "locate",
        "ranking_kind": "local_lexical_heuristic",
        "results": candidates,
        "ranked_candidates": total_candidates,
        "effective_query_terms": terms.len(),
        "source_of_truth_verified": false,
        "no_match_is_exhaustive": false,
        "snapshot_atomic": false
    });
    while response.to_string().len() > MAX_JSON_BYTES {
        let Some(items) = response["results"].as_array_mut() else { break; };
        if items.pop().is_none() { break; }
        response["ranking_truncated"] = json!(true);
    }
    response
}

#[cfg(test)]
#[path = "source_locate_tests.rs"]
mod tests;
