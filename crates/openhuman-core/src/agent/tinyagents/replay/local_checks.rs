//! Read-only local command-check evidence derived from the existing durable
//! TinyAgents event journal. This is not a replacement ledger or a completion
//! oracle: tool names and exit outcomes cannot prove full coverage of a task.
use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;
use tinyagents_harness::events::AgentEvent;
use tinyagents_harness::observability::AgentObservation;

/// Facts used for unit testing and by the typed AgentEvent adapter.
pub(super) enum LocalToolEvent<'a> {
    Started,
    Completed { metadata: Option<&'a Value>, error: bool },
    Failed,
}

#[derive(Debug, Serialize)]
pub(crate) struct LocalCheckReport {
    pub(crate) run_id: String,
    pub(crate) status: &'static str,
    pub(crate) last_check_offset: Option<u64>,
    pub(crate) last_possible_mutation_offset: Option<u64>,
    pub(crate) passed_checks: usize,
    pub(crate) failed_checks: usize,
    /// Safe category names only: command strings and arguments are never stored.
    pub(crate) unresolved_check_kinds: Vec<String>,
    /// Older receipts did not identify a check category and cannot prove a rerun.
    pub(crate) unattributed_failures: usize,
    pub(crate) unclassified_shell_calls: usize,
    pub(crate) check_after_last_known_mutation: bool,
    pub(crate) journal_complete: bool,
    pub(crate) task_verified: bool,
    pub(crate) scope: &'static str,
}

#[derive(Default)]
pub(super) struct LocalCheckAccumulator {
    latest_pass: Option<u64>,
    last_possible_mutation: Option<u64>,
    /// Last observed outcome by check family: cargo_test != cargo_fmt.
    /// No command contents, source paths, or user-supplied identifiers.
    checks_by_kind: BTreeMap<String, bool>,
    unattributed_failures: usize,
    passed: usize,
    failed: usize,
    unknown_shell: usize,
    previous_event_offset: Option<u64>,
    observed_any: bool,
    has_event_gaps: bool,
}

fn may_change_source(tool: &str) -> bool {
    // Unknown plugin/MCP tools may write, run shell or delegate. The journal
    // does not contain a trusted permission-level proof, so treating them as
    // read-only would let them leave a stale check marked fresh. Allow only a
    // deliberately small set of native, explicitly read-only tool names.
    !matches!(
        tool,
        "file_read" | "file_list" | "file_search" |
        "grep" | "glob" | "source_evidence" | "current_time" |
        "run_status" | "run_events" | "run_local_checks"
    )
}

fn supported_check_kind(kind: &str) -> bool {
    matches!(
        kind,
        "cargo_test" | "cargo_check" | "cargo_clippy" | "cargo_build" | "cargo_fmt" |
        "pytest" | "python_unittest" |
        "go_test" | "go_vet" | "go_build" | "node_test" |
        "pnpm_test" | "pnpm_lint" | "pnpm_build" | "vitest" |
        "npm_test" | "npm_build" | "npm_lint" | "npm_typecheck"
    )
}

fn trusted_local_fact(meta: Option<&Value>) -> Option<(bool, Option<&str>)> {
    let fact = meta?.get("openhuman_local_check_v1")?;
    if fact.get("recognized")?.as_bool() != Some(true)
        || fact.get("scope")?.as_str() != Some("local_check_only")
        || fact.get("task_verified")?.as_bool() != Some(false)
    {
        return None;
    }
    // This host-only envelope is versioned. A contradictory tuple such as
    // passed=true/status=not_executed cannot become positive evidence even
    // if some future tool or adapter accidentally writes that metadata.
    let kind = match fact.get("check_kind") {
        Some(Value::String(name)) if supported_check_kind(name) => Some(name.as_str()),
        Some(_) => return None, // an invented check category is not trusted
        None => None, // backward compatibility: no basis to clear old failures
    };
    let passed = fact.get("passed")?.as_bool()?;
    let status = fact.get("status")?.as_str()?;
    let exit = fact.get("exit_status")?.as_str()?;
    match (passed, status, exit) {
        (true, "passed", "zero") => Some((true, kind)),
        (false, "not_executed" | "failed_or_unobserved", "unknown") => Some((false, kind)),
        _ => None,
    }
}

impl LocalCheckAccumulator {
    pub(super) fn observe(&mut self, observation: &AgentObservation) {
        // Missing observations cannot be reconstructed into proof. The sink
        // can drop events under pressure, leaving nonconsecutive offsets.
        if !self.observed_any && observation.offset != 0 {
            self.has_event_gaps = true;
        }
        if let Some(previous) = self.previous_event_offset {
            if observation.offset != previous.saturating_add(1) {
                self.has_event_gaps = true;
            }
        }
        self.observed_any = true;
        self.previous_event_offset = Some(observation.offset);
        match &observation.event {
            AgentEvent::ToolStarted { tool_name, .. } => {
                self.record(observation.offset, tool_name, LocalToolEvent::Started);
            }
            AgentEvent::ToolCompleted {
                tool_name, metadata, error, ..
            } => self.record(
                observation.offset,
                tool_name,
                LocalToolEvent::Completed {
                    metadata: metadata.as_ref(),
                    error: error.is_some(),
                },
            ),
            AgentEvent::ToolFailed { tool_name, .. } => {
                self.record(observation.offset, tool_name, LocalToolEvent::Failed);
            }
            _ => {}
        }
    }

    pub(super) fn record(
        &mut self,
        offset: u64,
        tool_name: &str,
        event: LocalToolEvent<'_>,
    ) {
        if may_change_source(tool_name) {
            // A tool can modify a file even when its final outcome was failure.
            // Only a later trusted check can be newer than this observation.
            if !matches!(&event, LocalToolEvent::Completed { .. })
                || tool_name != "shell"
            {
                self.last_possible_mutation = Some(offset);
            }
        }
        if let LocalToolEvent::Completed { metadata, error } = event {
            if tool_name == "shell" {
                match trusted_local_fact(metadata) {
                    Some((true, kind)) if !error => {
                        self.latest_pass = Some(offset);
                        self.passed += 1;
                        if let Some(kind) = kind {
                            self.checks_by_kind.insert(kind.to_owned(), true);
                        }
                    }
                    Some((_, kind)) => {
                        self.failed += 1;
                        if let Some(kind) = kind {
                            self.checks_by_kind.insert(kind.to_owned(), false);
                        } else {
                            // A legacy failure with no category cannot be
                            // cleared by an unrelated later green command.
                            self.unattributed_failures += 1;
                        }
                    },
                    None => {
                        // No proven read-only classification: the command might
                        // have written files even when the process exited zero.
                        self.unknown_shell += 1;
                        self.last_possible_mutation = Some(offset);
                    }
                }
            }
        }
    }

    pub(super) fn finish(self, run_id: &str, fully_paged: bool) -> LocalCheckReport {
        let complete = fully_paged && !self.has_event_gaps;
        // A successful formatter cannot erase a failed test run. Only a
        // later successful check of the SAME safe family resolves its failure.
        let unresolved_check_kinds: Vec<String> = self.checks_by_kind.iter()
            .filter(|(_, passed)| !**passed)
            .map(|(kind, _)| kind.clone())
            .collect();
        let unresolved_failures = self.unattributed_failures > 0
            || !unresolved_check_kinds.is_empty();
        let fresh = complete && !unresolved_failures
            && self.latest_pass.is_some_and(|offset| {
                self.last_possible_mutation.is_none_or(|last| offset > last)
            });
        let status = if !complete {
            "partial_journal"
        } else if unresolved_failures {
            "failed_local_check"
        } else if self.latest_pass.is_some() && !fresh {
            "stale_local_check"
        } else if fresh {
            "local_check_observed"
        } else if self.failed > 0 {
            "failed_local_check"
        } else {
            "no_local_check"
        };
        LocalCheckReport {
            run_id: run_id.to_owned(),
            status,
            last_check_offset: self.latest_pass,
            last_possible_mutation_offset: self.last_possible_mutation,
            passed_checks: self.passed,
            failed_checks: self.failed,
            unresolved_check_kinds,
            unattributed_failures: self.unattributed_failures,
            unclassified_shell_calls: self.unknown_shell,
            check_after_last_known_mutation: fresh,
            journal_complete: complete,
            // A green local command never establishes post-commit CI, full
            // requirements coverage, or checks on all source mutation paths.
            task_verified: false,
            scope: "observed_journal_local_checks_only",
        }
    }
}

#[cfg(test)]
#[path = "local_checks_tests.rs"]
mod tests;
