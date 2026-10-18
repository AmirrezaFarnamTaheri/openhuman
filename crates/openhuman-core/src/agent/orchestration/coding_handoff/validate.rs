//! Bounded report validation, with no filesystem or process access.

use std::collections::HashSet;

use super::types::{CheckStatus, CodingHandoff};

pub(super) const MAX_RECORDS: usize = 32;
pub(super) const MAX_CHECKS: usize = 16;
pub(super) const MAX_TEXT_BYTES: usize = 2048;
pub(super) const MAX_PATH_BYTES: usize = 1024;
pub(super) const MAX_SYMBOLS: usize = 16;

pub(super) fn validate(packet: &CodingHandoff) -> Result<(), String> {
    if packet.version != 1 {
        return Err("unsupported version; expected 1".into());
    }
    text("objective", &packet.objective, MAX_TEXT_BYTES, true)?;
    count("source_refs", packet.source_refs.len(), MAX_RECORDS)?;
    unique_paths(packet.source_refs.iter().map(|item| item.path.as_str()))?;
    for source in &packet.source_refs {
        path(&source.path)?;
        revision(&source.revision)?;
        count(
            "selected_symbols",
            source.selected_symbols.len(),
            MAX_SYMBOLS,
        )?;
        for symbol in &source.selected_symbols {
            text("selected symbol", symbol, 256, false)?;
        }
    }
    count("changed_files", packet.changed_files.len(), MAX_RECORDS)?;
    unique_paths(packet.changed_files.iter().map(|item| item.path.as_str()))?;
    for changed in &packet.changed_files {
        path(&changed.path)?;
        if changed.previous_revision.is_none() && changed.revision.is_none() {
            return Err("changed files need a previous or current revision".into());
        }
        for value in [
            changed.previous_revision.as_deref(),
            changed.revision.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            revision(value)?;
        }
    }
    for (field, items) in [
        ("intended_edits", &packet.intended_edits),
        ("risks", &packet.risks),
    ] {
        count(field, items.len(), MAX_RECORDS)?;
        for item in items {
            text(field, item, MAX_TEXT_BYTES, true)?;
        }
    }
    if let Some(next) = &packet.next_action {
        text("next_action", next, MAX_TEXT_BYTES, true)?;
    }
    count("checks", packet.checks.len(), MAX_CHECKS)?;
    for check in &packet.checks {
        text("command", &check.command, MAX_TEXT_BYTES, true)?;
        text("cwd", &check.cwd, MAX_PATH_BYTES, false)?;
        match check.status {
            CheckStatus::Passed if check.exit_code != Some(0) => {
                return Err("a reported passed check needs exit_code 0".into());
            }
            CheckStatus::Failed if check.exit_code.is_none_or(|code| code == 0) => {
                return Err("a reported failed check needs a nonzero exit_code".into());
            }
            _ => {}
        }
        count(
            "check source_revisions",
            check.source_revisions.len(),
            MAX_RECORDS,
        )?;
        unique_paths(check.source_revisions.iter().map(|item| item.path.as_str()))?;
        for source in &check.source_revisions {
            path(&source.path)?;
            revision(&source.revision)?;
        }
        if let Some(summary) = &check.summary {
            text("check summary", summary, MAX_TEXT_BYTES, true)?;
        }
    }
    Ok(())
}

fn count(field: &str, actual: usize, max: usize) -> Result<(), String> {
    if actual > max {
        return Err(format!("{field} exceeds {max} entries"));
    }
    Ok(())
}

fn unique_paths<'a>(paths: impl Iterator<Item = &'a str>) -> Result<(), String> {
    let mut seen = HashSet::new();
    for path in paths {
        if !seen.insert(path) {
            return Err("duplicate source path within one list".into());
        }
    }
    Ok(())
}

fn text(field: &str, value: &str, max: usize, multiline: bool) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > max {
        return Err(format!(
            "{field} must be nonblank and at most {max} UTF-8 bytes"
        ));
    }
    if value.chars().any(|character| {
        character.is_control() && !(multiline && matches!(character, '\n' | '\r' | '\t'))
    }) {
        return Err(format!("{field} contains unsupported control characters"));
    }
    Ok(())
}

fn path(value: &str) -> Result<(), String> {
    text("source path", value, MAX_PATH_BYTES, false)?;
    if value.contains(['\\', ':'])
        || value
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("source paths must be portable workspace-relative file paths".into());
    }
    Ok(())
}

fn revision(value: &str) -> Result<(), String> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(
            "revisions must be the 64 lowercase hex digits returned by structural tools".into(),
        );
    }
    Ok(())
}
