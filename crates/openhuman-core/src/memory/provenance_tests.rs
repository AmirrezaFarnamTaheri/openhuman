use super::*;
#[test]
fn locally_authored_namespaces_are_trusted() {
    for ns in [
        "working", "agent", "local", "core", "global", "default", "user",
    ] {
        assert!(
            !connector_provenance_policy().is_potentially_untrusted(Some(ns), "k"),
            "namespace '{ns}' must be trusted"
        );
    }
}

#[test]
fn prefixed_subspaces_are_trusted() {
    for ns in ["working.user.123", "agent.session.foo", "tree.discord.456"] {
        assert!(
            !connector_provenance_policy().is_potentially_untrusted(Some(ns), "k"),
            "namespace '{ns}' must be trusted"
        );
    }
}

#[test]
fn unknown_namespace_is_untrusted() {
    // Default-deny — any unrecognised namespace flips to untrusted so
    // a future connector that lands without explicit allowlisting is
    // wrapped by default.
    assert!(connector_provenance_policy().is_potentially_untrusted(Some("scraped"), "k"));
    assert!(connector_provenance_policy().is_potentially_untrusted(Some("composio"), "k"));
}

#[test]
fn connector_key_prefix_is_untrusted_even_without_namespace() {
    assert!(connector_provenance_policy().is_potentially_untrusted(None, "chat:discord:42"));
    assert!(connector_provenance_policy().is_potentially_untrusted(None, "gmail:thread:xyz"));
    assert!(connector_provenance_policy().is_potentially_untrusted(None, "notion:page:abc"));
}

#[test]
fn no_namespace_plain_key_is_trusted() {
    // No namespace + no connector prefix = locally authored by
    // default (the bare-key tooling path doesn't reach this code).
    assert!(!connector_provenance_policy().is_potentially_untrusted(None, "user_pref:theme"));
}

#[test]
fn peer_imported_source_prefixes_are_classified_as_external() {
    let policy = connector_provenance_policy();
    for key in [
        "repo:code:readme",
        "source:sha256:abc",
        "code:src/lib.rs",
        "github:project/repo",
        "gitlab:group/project",
        "conduit:graph-node",
        "jev:ranked-evidence",
        "typesafe:decision",
        "9router:model-response",
    ] {
        assert!(policy.is_potentially_untrusted(None, key), "{key}");
    }
}

#[test]
fn peer_source_policy_preserves_local_user_namespace() {
    let policy = connector_provenance_policy();
    assert!(!policy.is_potentially_untrusted(Some("user"), "favorite-editor"));
    assert!(!policy.is_potentially_untrusted(Some("working"), "current-project"));
}
