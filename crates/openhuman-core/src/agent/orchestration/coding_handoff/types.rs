//! The opt-in, host-owned coding handoff report. These are reported facts,
//! never authority, executable instructions, or verifier evidence.

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CodingHandoff {
    #[serde(rename = "type")]
    pub kind: HandoffKind,
    pub version: u32,
    pub objective: String,
    #[serde(default)]
    pub source_refs: Vec<SourceRef>,
    #[serde(default)]
    pub changed_files: Vec<ChangedFile>,
    #[serde(default)]
    pub intended_edits: Vec<String>,
    #[serde(default)]
    pub checks: Vec<Check>,
    #[serde(default)]
    pub risks: Vec<String>,
    #[serde(default)]
    pub next_action: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(super) enum HandoffKind {
    #[serde(rename = "coding_handoff")]
    CodingHandoff,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceRef {
    pub path: String,
    pub revision: String,
    #[serde(default)]
    pub selected_symbols: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceRevision {
    pub path: String,
    pub revision: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ChangedFile {
    pub path: String,
    /// Absent for a new file; the current revision is absent for a deletion.
    #[serde(default)]
    pub previous_revision: Option<String>,
    #[serde(default)]
    pub revision: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Check {
    pub command: String,
    /// Exact observed working directory. It is displayed, never resolved.
    pub cwd: String,
    #[serde(default)]
    pub status: CheckStatus,
    #[serde(default)]
    pub exit_code: Option<i32>,
    #[serde(default)]
    pub source_revisions: Vec<SourceRevision>,
    #[serde(default)]
    pub summary: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CheckStatus {
    Passed,
    Failed,
    Skipped,
    Cancelled,
    #[default]
    Unverified,
}
