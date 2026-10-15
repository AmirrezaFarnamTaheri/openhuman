//! Result mapping and child-environment helpers for the `shell` tool.

use std::time::Duration;
use tinytools::ToolResult;

/// Exit status coreutils `timeout` returns when its own limit expires.
pub(super) const COMMAND_TIMEOUT_EXIT_CODE: i32 = 124;
/// Appended to an exit-124 failure: the command's own `timeout` fired, not the
/// tool's (#6953). Without it, `exit code 124 [stdout] ok` reads like a bug.
pub(super) const COMMAND_TIMEOUT_NOTE: &str = "[exit 124: the command's own `timeout` limit expired; the shell tool's timeout_secs did not fire]";

/// The result when the tool's own `timeout_secs` deadline killed the command.
pub(super) fn tool_timeout_message(secs: u64) -> String {
    format!("Command timed out after {secs}s and was killed: the shell tool's timeout_secs limit fired.")
}

/// [`tinytools::command_failure`], plus [`COMMAND_TIMEOUT_NOTE`] on exit 124.
pub(super) fn command_failure(code: Option<i32>, stdout: &str, stderr: &str) -> ToolResult {
    let mut failure = tinytools::command_failure(code, stdout, stderr);
    if code == Some(COMMAND_TIMEOUT_EXIT_CODE) {
        tracing::debug!("[shell] exit 124: attributing it to the command's own timeout");
        failure.content.push(tinytools::ToolContent::Text {
            text: COMMAND_TIMEOUT_NOTE.to_string(),
        });
    }
    failure
}

/// Map a finished sandbox or module execution to the tool result.
///
/// The same exit-code and both-streams surfacing everywhere (#4095); the `-1`
/// sentinel renders as a signal.
pub(super) fn exec_result(
    result: anyhow::Result<crate::sandbox::SandboxExecResult>,
    effective: Duration,
) -> ToolResult {
    match result {
        Ok(result) if result.timed_out => {
            ToolResult::error(tool_timeout_message(effective.as_secs()))
        }
        Ok(result) if result.success() => {
            if result.stderr.is_empty() {
                ToolResult::success(result.stdout)
            } else {
                ToolResult::success(format!("{}\n[stderr]\n{}", result.stdout, result.stderr))
            }
        }
        Ok(result) => command_failure(
            tinytools::sandbox_exit_code(result.exit_code),
            &result.stdout,
            &result.stderr,
        ),
        Err(e) => ToolResult::error(format!("Sandbox execution failed: {e}")),
    }
}

/// Python's text-encoding defaults for a child, unless the parent sets its own.
/// Under `cmd.exe` the console code page is a legacy one (cp1252, cp437), so a
/// script printing anything outside it dies with `UnicodeEncodeError`. Both are
/// no-ops for a non-Python command. Native runs get the same from TinyBox's
/// `python_utf8` request flag; this serves the Docker and sandbox-policy paths,
/// which forward their environment as plain variables.
pub(super) const PYTHON_UTF8_DEFAULTS: &[(&str, &str)] =
    &[("PYTHONUTF8", "1"), ("PYTHONIOENCODING", "utf-8")];

pub(super) fn python_utf8_env(
    lookup: impl Fn(&str) -> Option<std::ffi::OsString>,
) -> Vec<(&'static str, std::ffi::OsString)> {
    PYTHON_UTF8_DEFAULTS
        .iter()
        .map(|(name, default)| {
            let value = lookup(name).unwrap_or_else(|| (*default).into());
            (*name, value)
        })
        .collect()
}

/// The host environment a Docker container receives: the sandbox allowlist from
/// the parent plus the Python encoding defaults. `None` when a value is not UTF-8.
#[cfg(feature = "modules")]
pub(super) fn container_environment() -> Option<std::collections::BTreeMap<String, String>> {
    let mut env = std::collections::BTreeMap::new();
    let inherited = crate::sandbox::ops::SANDBOX_ENV_PASSTHROUGH
        .iter()
        .filter_map(|name| {
            tinytools_std::timeout::CommandEnvironment::var_os(name).map(|value| (*name, value))
        })
        .chain(python_utf8_env(
            tinytools_std::timeout::CommandEnvironment::var_os,
        ));
    for (name, value) in inherited {
        env.insert(name.to_owned(), value.into_string().ok()?);
    }
    Some(env)
}

/// Recognize only a simple top-level test/build/lint invocation whose OS
/// process exit is propagated by the shell tool. Deliberately conservative:
/// pipes, redirections, quoting, shell expansion and chained commands are
/// excluded because a green wrapper may hide a failed check.
/// A stable, low-cardinality command *category*, never a command string.
/// This lets the journal distinguish "tests failed then formatter passed"
/// from "tests failed then tests were rerun successfully" without storing
/// arguments, paths, stdout, stderr, credentials or hashes.
fn verification_kind(command: &str) -> Option<&'static str> {
    let source = command.trim();
    if source.is_empty()
        || source.chars().any(|c| {
            matches!(c, '|' | '&' | ';' | '<' | '>' | '$' | '\x60' | '\\' | '\'' | '"' | '#' | '\n' | '\r')
        })
    {
        return None;
    }
    let args: Vec<&str> = source.split_whitespace().collect();
    if args.iter().any(|arg| {
        matches!(
            *arg,
            "-h" | "-V" | "--help" | "--version" | "--list" |
            "-list" | "--collect-only" | "--no-run" | "--dry-run" |
            "--if-present" | "--passWithNoTests"
        )
    }) {
        return None;
    }
    match args.as_slice() {
        ["cargo", "test", ..] => Some("cargo_test"),
        ["cargo", "check", ..] => Some("cargo_check"),
        ["cargo", "clippy", ..] => Some("cargo_clippy"),
        ["cargo", "build", ..] => Some("cargo_build"),
        ["cargo", "fmt", "--check", ..] => Some("cargo_fmt"),
        ["pytest", ..] | ["python" | "python3", "-m", "pytest", ..] => Some("pytest"),
        ["python" | "python3", "-m", "unittest", ..] => Some("python_unittest"),
        ["go", "test", ..] => Some("go_test"),
        ["go", "vet", ..] => Some("go_vet"),
        ["go", "build", ..] => Some("go_build"),
        ["node", "--test", ..] => Some("node_test"),
        ["pnpm", "test", ..] => Some("pnpm_test"),
        ["pnpm", "lint", ..] => Some("pnpm_lint"),
        ["pnpm", "build", ..] => Some("pnpm_build"),
        ["pnpm", "exec", "vitest", "run", ..] => Some("vitest"),
        ["npm", "test", ..] => Some("npm_test"),
        ["npm", "run", "test", ..] => Some("npm_test"),
        ["npm", "run", "build", ..] => Some("npm_build"),
        ["npm", "run", "lint", ..] => Some("npm_lint"),
        ["npm", "run", "typecheck", ..] => Some("npm_typecheck"),
        _ => None,
    }
}

#[cfg(test)]
pub(super) fn is_verification_command(command: &str) -> bool {
    verification_kind(command).is_some()
}

/// Host-only observation integrated into the existing TinyAgents ToolCompleted
/// event and durable journal, NOT the model's tool-result text. No command
/// string, CLI argument, output, key, or potentially sensitive hash is stored.
/// This is a *local check observation*, not task verification/CI freshness.
pub(super) fn append_verification_receipt(
    result: &mut ToolResult,
    command: &str,
    authorized: bool,
) {
    let Some(check_kind) = verification_kind(command) else {
        return;
    };
    let passed = authorized && !result.is_error;
    let status = if !authorized {
        "not_executed"
    } else if passed {
        "passed"
    } else {
        "failed_or_unobserved"
    };
    let fact = serde_json::json!({
        "recognized": true,
        "check_kind": check_kind,
        "passed": passed,
        "status": status,
        "exit_status": if passed { "zero" } else { "unknown" },
        "scope": "local_check_only",
        "task_verified": false
    });
    match result.metadata.as_mut() {
        Some(serde_json::Value::Object(existing)) => {
            existing.insert("openhuman_local_check_v1".to_owned(), fact);
        }
        Some(_) => {
            // Preserve unrelated non-object metadata rather than overwriting it.
        }
        None => {
            result.metadata = Some(serde_json::json!({"openhuman_local_check_v1":fact}));
        }
    }
}

#[cfg(test)]
#[path = "shell_verification_tests.rs"]
mod verification_tests;
