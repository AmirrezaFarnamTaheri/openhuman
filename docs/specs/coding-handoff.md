# Coding handoff context, version 1

The host accepts a small coding report through the existing optional `context`
string on worker calls. This convention carries an objective, source revisions,
selected symbols, edits and reported validation commands between workers. It
does not introduce an agent, change delegation schemas or grant permissions.

## Opting in

After optional leading whitespace, the context must begin with the exact,
case-sensitive marker `[coding_handoff]`, followed by one JSON object. The
object requires `"type": "coding_handoff"`, `"version": 1` and a nonblank
`objective`. A marker in the middle of ordinary prose has no special meaning.
Unmarked strings, including ordinary JSON, retain their existing handling.

```text
[coding_handoff]
{"type":"coding_handoff","version":1,"objective":"Review the parser change","source_refs":[{"path":"src/parser.rs","revision":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","selected_symbols":["Parser::parse"]}],"checks":[{"command":"cargo test --locked --lib","cwd":"/project","status":"unverified"}],"next_action":"Inspect the current revision and run the focused test"}
```

The revision in this example is illustrative. Producers must copy actual
content revisions from the source inspection or edit receipt; branch names and
commit abbreviations are not content revisions.

## Fields and limits

All limits count UTF-8 bytes. Both the marked input and the rendered report
must fit 32,768 bytes. Unknown object fields, duplicate JSON fields, wrong
types, trailing JSON and unsupported versions are rejected. Records must be
objects and enum values must be strings; positional record arrays and
object-encoded enum variants are rejected. The parser never
echoes untrusted field names or values in an error.

| Field | Contract |
| --- | --- |
| `type` | Required string, exactly `coding_handoff`. |
| `version` | Required integer, exactly `1`. |
| `objective` | Required nonblank text, at most 2,048 bytes. |
| `source_refs` | Optional array, at most 32 records containing `path`, `revision` and optional `selected_symbols`. |
| `selected_symbols` | At most 16 nonblank identifiers or selectors per source reference, each at most 256 bytes. These are reported selectors, not resolved symbols. |
| `changed_files` | Optional array, at most 32 records containing `path` and at least one of `previous_revision` or `revision`. Omit the previous revision for creation and the current revision for deletion. |
| `intended_edits` | Optional array of at most 32 nonblank texts, each at most 2,048 bytes. |
| `checks` | Optional array, at most 16 reported checks, described below. |
| `risks` | Optional array of at most 32 nonblank texts, each at most 2,048 bytes. |
| `next_action` | Optional nonblank text, at most 2,048 bytes. |

Source paths are portable, workspace-relative file paths of at most 1,024
bytes. Empty components, `.` or `..` components, absolute paths, backslashes,
colons and control characters are rejected. Each source list, changed-file
list and check's source-revision list must have unique paths. Revisions are
exactly 64 lowercase hexadecimal digits. Validation does not open a path or
test whether a reported revision is current.

Each check contains:

| Field | Contract |
| --- | --- |
| `command` | Required nonblank text, at most 2,048 bytes; retained exactly. |
| `cwd` | Required nonblank text, at most 1,024 bytes; the exact reported working directory, retained without path resolution. Absolute and platform-specific paths are allowed here because this field is evidence text. |
| `status` | `passed`, `failed`, `skipped`, `cancelled` or `unverified`; defaults to `unverified`. |
| `exit_code` | Optional signed 32-bit integer. `passed` requires `0`; `failed` requires a nonzero value. |
| `source_revisions` | Optional array of at most 32 `{path, revision}` records. |
| `summary` | Optional nonblank text, at most 2,048 bytes. |

Prose and commands may contain tab and line-break characters. Paths, working
directories and symbol selectors may not contain control characters.

## Admission and reuse

The host validates marked contexts before deriving a child identity, creating
a worker thread, registering asynchronous work, cancelling or steering a
reusable worker, or staging a parallel worktree. The entire parallel batch is
validated before any task is staged; the typed batch entry point uses the same
check as the JSON tool entry point. Direct and resumed subagent calls also
validate before lifecycle coalescing or persistence. The direct execution leaf
retains its existing first dispatch-budget check, then validates context before
parent lookup or execution.

Accepted reports are rendered with a fixed untrusted-report notice and
canonical JSON. Revalidating a rendered report does not wrap it again. The
asynchronous worker's derived task key uses that prepared context, so JSON
whitespace differences do not create distinct task identities. An explicitly
supplied task key keeps its existing meaning.

The configured `delegate` tool applies the same validation before looking up a
model or enforcing an action; it retains its existing trimming behavior for
ordinary context. The compact archetype and `delegate_to` schemas remain
unchanged: their task prompt may carry prose, and they do not gain a new
context parameter.

## Evidence and authority

Every rendered report begins with `Verification: unverified`. A consistent
`passed` entry remains a caller's claim, even when it includes an exit code and
source revisions. Missing, skipped and cancelled checks cannot become a pass.
The receiving worker must inspect the current source and rerun relevant checks
after writes. This convention never populates finish-verification state.

Commands are displayed as data and are never executed during admission. Paths
are never dereferenced or imported, and the report is not silently saved as an
artifact. Ordinary tool policy, filesystem authorization, sandboxing,
cancellation and lifecycle rules remain authoritative when the worker later
acts. The marker is an input-format convention, not a trust boundary against
malicious prose.

## Verification

The helper's adjacent tests cover legacy byte preservation, canonical rendering,
UTF-8 bounds, malformed contracts, reported exit codes, duplicate paths, source
revisions and size limits. Consumer tests cover direct child prompt delivery,
early refusal, asynchronous identity/reuse, typed parallel admission and the
configured delegate. The `coding_handoff` CLI integration target exercises the
actual production subagent lifecycle and tool entry points with a scripted
model and temporary workspaces; it also compiles the helper's adjacent tests
from the production source path. No test calls a live model service.
