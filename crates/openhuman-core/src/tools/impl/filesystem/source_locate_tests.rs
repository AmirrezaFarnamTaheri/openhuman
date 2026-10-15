//! Regression contract for bounded, no-egress source location.
use super::{rank_sources, SourceDocument};
use serde_json::Value;
use sha2::Digest;

fn src(path: &str, text: &str) -> SourceDocument {
    SourceDocument {
        path: path.to_owned(),
        source_hash: format!("{:x}", sha2::Sha256::digest(text.as_bytes())),
        content: text.to_owned(),
    }
}

#[test]
fn favors_relevant_symbols_and_paths_over_unrelated_long_documents() {
    let documents = vec![
        src("docs/readme.md", "User guide. Login is a process. The actual session code is elsewhere."),
        src("src/auth/session_manager.ts",
            "export function assertSessionFresh(session: Session) {\n  return session.expiresAt > Date.now();\n}\n"),
        src("src/billing/invoices.ts", "export function chargeInvoice() { return 1; }\n"),
    ];
    let found = rank_sources(&documents, "where is session freshness checked", 5);
    assert_eq!(found["results"][0]["path"], "src/auth/session_manager.ts");
    assert_eq!(found["results"][0]["evidence"][0]["line"], 1);
    assert_eq!(found["results"][0]["evidence"][0]["text"],
        "export function assertSessionFresh(session: Session) {");
    assert_eq!(found["ranking_kind"], "local_lexical_heuristic");
    assert_eq!(found["source_of_truth_verified"], false);
    assert_eq!(found["results"][0]["source_hash"], documents[1].source_hash);
}

#[test]
fn token_boundaries_do_not_treat_partial_words_as_matches() {
    let docs = vec![
        src("src/superuser.ts", "const superuser = true;\n"),
        src("src/user.ts", "const user = true;\n"),
    ];
    let found = rank_sources(&docs, "user", 5);
    assert_eq!(found["results"].as_array().unwrap().len(), 1);
    assert_eq!(found["results"][0]["path"], "src/user.ts");
}

#[test]
fn path_and_camel_case_tokens_are_indexed_and_ties_are_deterministic() {
    let docs = vec![
        src("z/authSession.ts", "export const authSession = true;\n"),
        src("a/auth_session.ts", "export const authSession = true;\n"),
    ];
    let a = rank_sources(&docs, "auth session", 5);
    let b = rank_sources(&docs.iter().rev().cloned().collect::<Vec<_>>(), "auth session", 5);
    assert_eq!(a["results"], b["results"]);
    assert_eq!(a["results"][0]["path"], "a/auth_session.ts");
}

#[test]
fn missing_matches_are_declared_unknown_not_a_negative_finding() {
    let docs = vec![src("src/unrelated.rs", "fn something() {}\n")];
    let found = rank_sources(&docs, "payment settlement", 3);
    assert_eq!(found["results"], serde_json::json!([]));
    assert_eq!(found["no_match_is_exhaustive"], false);
    assert_eq!(found["source_of_truth_verified"], false);
}

#[test]
fn stopword_queries_are_rejected_without_fabricating_candidates() {
    let found = rank_sources(&[src("src/lib.rs", "fn main() {}\n")], "where is the", 5);
    assert_eq!(found["results"], serde_json::json!([]));
    assert_eq!(found["effective_query_terms"], 0);
}

#[test]
fn result_is_bounded_and_excerpt_is_valid_unicode() {
    let line = format!("fn reveal() {{ {} }}", "🙂".repeat(512));
    let docs: Vec<_> = (0..32)
        .map(|i| src(&format!("src/session_{i:02}.rs"), &format!("{line}\n{line}\n{line}\n{line}\n")))
        .collect();
    let found = rank_sources(&docs, "reveal", 5);
    assert!(found.to_string().len() <= 26_000);
    assert!(found["results"].as_array().unwrap().len() <= 5);
    for item in found["results"].as_array().unwrap() {
        assert!(item["evidence"].as_array().unwrap().len() <= 3);
        for entry in item["evidence"].as_array().unwrap() {
            assert!(entry["text"].as_str().unwrap().chars().count() <= 160);
        }
    }
}

#[test]
fn limit_is_clamped_and_zero_cannot_be_unbounded() {
    let docs: Vec<_> = (0..32)
        .map(|i| src(&format!("src/check_{i:02}.ts"), "export const check = true;\n"))
        .collect();
    assert_eq!(rank_sources(&docs, "check", 0)["results"].as_array().unwrap().len(), 1);
    assert_eq!(rank_sources(&docs, "check", 500)["results"].as_array().unwrap().len(), 5);
}
