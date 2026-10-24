//! Host side of the optional wall-clock deadline for a top-level agent turn.
//! The deadline itself, including the close budget, is
//! `tinyagents_harness::turn_deadline::TurnDeadline`.
//!
//! No total-turn wall-clock deadline is applied by default. A user can opt in
//! with `OPENHUMAN_WEB_TURN_TIMEOUT_SECS` or
//! `OPENHUMAN_AGENT_TURN_TIMEOUT_SECS`; `0` disables the cap. When configured,
//! the web guard supplies a backstop and the harness winds down before it so
//! the session can return a grounded close.
//!
//! This is separate from the web-turn idle guard, which only ends a run after
//! it stops making progress with no model or tool call in flight.

use std::time::Duration;

/// Environment override for the backstop, in seconds; `0` disables it.
pub const TURN_BACKSTOP_ENV: &str = "OPENHUMAN_WEB_TURN_TIMEOUT_SECS";

/// Pure core of [`configured_backstop`]: an absent, unparseable, or zero value
/// leaves total turn time unbounded.
pub fn parse_web_backstop(env_value: Option<&str>) -> Option<Duration> {
    env_value
        .and_then(|v| v.trim().parse::<u64>().ok())
        .filter(|secs| *secs > 0)
        .map(Duration::from_secs)
}

/// The backstop in force for this process (env [`TURN_BACKSTOP_ENV`]).
pub fn configured_backstop() -> Option<Duration> {
    let configured = std::env::var(TURN_BACKSTOP_ENV)
        .ok()
        .or_else(|| std::env::var("OPENHUMAN_AGENT_TURN_TIMEOUT_SECS").ok());
    parse_web_backstop(configured.as_deref())
}

#[cfg(test)]
#[path = "backstop_tests.rs"]
mod tests;
