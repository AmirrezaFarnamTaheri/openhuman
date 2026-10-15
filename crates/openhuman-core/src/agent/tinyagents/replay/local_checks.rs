//! Read-only local command-check evidence derived from the existing durable
//! TinyAgents event journal. This is not a replacement ledger or a completion
//! oracle: tool names and exit outcomes cannot prove full coverage of a task.
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
    passed: usize,
    failed: usize,
    unknown_shell: usize,
    previous_event_offset: Option<u64>,
    observed_any: bool,
    has_event_gaps: bool,
}

fn may_change_source(tool: &str) -> bool {
    matches!(
        tool,
        "file_write" | "file_edit" | "edit_file" | "apply_patch" |
        "file_delete" | "file_move" | "git_operations" |
        "run_workflow" | "delegate" | "spawn_async_subagent" |
        "spawn_worker_thread" | "spawn_parallel_agents" |
        "continue_subagent" | "steer_subagent" | "shell"
    )
}

fn trusted_local_fact(meta: Option<&Value>) -> Option<bool> {
    let fact = meta?.get("openhuman_local_check_v1")?;
    if fact.get("recognized")?.as_bool() != Some(true)
        || fact.get("scope")?.as_str() != Some("local_check_only")
        || fact.get("task_verified").and_then(Value::as_bool) == Some(true)
    {
        return None;
    }
    let passed = fact.get("passed")?.as_bool()?;
    if passed && fact.get("exit_status")?.as_str() != "zero" {
        return None;
    }
    Some(passed)
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
                    Some(true) if !error => {
                        self.latest_pass = Some(offset);
                        self.passed += 1;
                    }
                    Some(_) => self.failed += 1,
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
        let fresh = complete
            && self.latest_pass.is_some_and(|offset| {
                self.last_possible_mutation.is_none_or(|last| offset > last)
            });
        let status = if !complete {
            "partial_journal"
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
