//! The memory block: what long-term memory knows about the user, rendered as
//! one delimited section for the end of an agent's system prompt.
//!
//! The session host (`agent/session_host/runtime_session/memory_block.rs`)
//! decides *when* a block is rendered: only when a transcript generation
//! starts, so the system prompt stays byte-identical, and the provider's
//! prefix cache warm, for the rest of the generation. This module only reads
//! and renders.
//!
//! What goes in: the beliefs the engine built (most confident first), then
//! the durable learnings under the identity's root (newest first), each one
//! line, deduplicated, until the next line would take the block past its
//! token budget ([`estimate_tokens`], TinyMemory's own estimator). Nothing
//! known renders no block at all, and memory switched off reads as nothing
//! known; an engine failure is an error the caller logs and survives.

use tinymemory_api::{BELIEF_TAG, BeliefsRequest, Hit, ListRequest, Reach, Result};
use tinymemory_tools::layout::identity::ResolvedIdentity;
use tinymemory_tools::recall::estimate_tokens;

use crate::config::Config;

/// Opens the system message that carries the memory block.
pub(crate) const MEMORY_BLOCK_MARKER: &str = "<memory>\n";

/// Closes the memory block.
pub(crate) const MEMORY_BLOCK_CLOSING: &str = "</memory>";

/// Most beliefs read for one block.
const BELIEFS_LIMIT: usize = 50;

/// Most learnings read for one block.
const FACTS_LIMIT: usize = 100;

/// The block for `beliefs` then `facts`, within `max_tokens`; `None` when
/// not one line fits.
pub(crate) fn render(_beliefs: &[Hit], _facts: &[Hit], _max_tokens: u32) -> Option<String> {
    None
}

/// Reads what `identity` knows about the user and renders it
/// ([`render`]). `Ok(None)` when nothing is known or memory is off.
///
/// # Errors
///
/// The engine's own failure to list beliefs or learnings.
pub(crate) async fn read(
    _config: &Config,
    _identity: &ResolvedIdentity,
    _max_tokens: u32,
) -> Result<Option<String>> {
    let _ = (BELIEFS_LIMIT, FACTS_LIMIT, BELIEF_TAG, estimate_tokens(""));
    let _ = BeliefsRequest::new;
    let _ = ListRequest::new;
    let _ = Reach::subtree;
    Ok(None)
}

#[cfg(test)]
#[path = "memory_block_tests.rs"]
mod tests;
