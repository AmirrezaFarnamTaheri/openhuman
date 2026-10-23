# Conversation organization and run panel

Status: approved by the user on 2026-10-23; the new run panel is not implemented.

## Intent

Help the operator understand, direct, and resume work without turning chat into
a dashboard they must constantly manage. The existing left sidebar is the
**conversation dock**. A separate right-hand **run panel** belongs to the selected
conversation. These names and responsibilities must remain distinct.

The user requested project/custom categories; nested agents; visible progress,
tools, skills, edits, communications and memory changes; direct child messaging;
working-directory and authorization controls; and durable continuation after
interruptions. Time and token limits are optional, never implicit session timers.

## Approach

Three visual explorations considered an overview-first dock, an agent workbench,
and a narrow navigation rail with an expanding inspector. The proposed direction
combines the agent hierarchy with a collapsible inspector. The user approved
this written design on 2026-10-23; runtime behavior still requires implementation.

Use one panel with four views: Overview, Agents, Activity, Controls. Do not add
independent floating panels for every feature. On narrower windows it becomes
the existing accessible Sheet; on desktop it occupies layout space without
covering chat. Preserve the chat's readable width and composer access. The
builder chooses precise responsive breakpoints from measured content widths.

## Conversation dock on the left

- Show top-level conversations grouped by full working-directory identity or
  an explicit custom category. Custom assignment takes precedence. Pins remain
  first; archive is a separate view and Delete remains available.
- Group headings expand/collapse. Similar folder names expose their full paths.
  Windows path spelling normalizes case and separators; Unix paths retain case.
- Subagent conversations never appear as unrelated top-level chats. An optional
  disclosure beneath a parent reveals children and their descendants.
- Changing selection opens that conversation's state; late events from a prior
  selection must not change the active panel or message recipient.

## Panel layout and default experience

The header contains the project/folder, a textual run status, and the relevant
Pause/Continue/Stop action. Show one current-action summary and an attention
count for approvals, questions and recoverable failures. Context occupancy may
use a bar; do not invent a percentage for overall task completion.

Overview is the default: current task, plan steps, blockers, and queued work.
Successful low-level events are grouped; questions and pending authorizations
stay visible. Completed runs show the outcome with collapsed details. Empty
conversations explain that run information appears after sending a message.

Agents shows the task hierarchy with stable identities, task summaries and
plain statuses. Selecting a child opens its own transcript and message composer
inside the panel. The main chat composer continues to address the parent.

Activity is an ordered, filterable record of tool/skill calls, edits, agent
communications, attachments and memory changes. Reuse the existing tool and
diff renderers. Summarize actions first; inputs, full output and technical
diagnostics are expandable. Preserve line breaks in search/terminal output.
Memory entries describe confirmed writes and their scope; a model's claim alone
does not establish that memory was saved.

Controls groups directory, sandbox, authorizations, context/compaction and optional
budgets. Advanced fields start collapsed. Show current effective values, scope
and when each change takes effect. Global defaults and this-conversation changes
are separate explicit choices. No automatic refresh may overwrite an edited form.

## Direct agent communication

- A composer says exactly whom it addresses, e.g. `Message Vision`. Bind its
  draft and submission to a stable task/session identity and parent conversation.
- Running recipients accept steering at the next safe boundary, after the
  current tool batch. Queued follow-ups remain distinct from steering.
- Waiting recipients accept an answer and continue that same task. Interrupted
  recipients offer `Continue with this message`. Completed reusable recipients
  accept a follow-up without silently creating an unrelated worker.
- Persist the message and a unique submission id before acknowledging it. Show
  queued, delivered or failed; transport reconnection must not duplicate delivery.
  `Delivered` means admission to the recipient's durable input, not model agreement.
- Preserve unsent text on failure. A status/selection change must never reroute
  a draft to the parent or a different child.
- Communication rows identify sender, recipient and delivery state. Hide repeated
  transport events; retain the actual message history and parent-child links.

## Interruption and continuation

Control scope is explicit in the action label. Header actions are `Pause run`,
`Continue run` and `Stop run`, targeting the selected root run and its attached
descendant tree. An agent-specific action names that agent and targets its
subtree. Unrelated parallel roots are unaffected. Pause is cooperative: finish
the current tool batch, show Pausing until settled, and retain queued work and
messages. Stop requests cancellation immediately and records uncertain effects.

Persist a pause-group identifier with each affected task. Continue run resumes
the root and only descendants paused by that group; children paused separately
stay paused. Queued children retain their queue position. An application restart
alone never resumes anything. After a crash, Continue reconstructs the selected
run from committed state, reconciling its child records before admitting any
unfinished child work. Do not spawn replacement children for existing identities.

Every agent task needs a durable session identity, transcript generation chain,
task status, pending inbox and recoverable checkpoint. Persist progress at safe
boundaries, including the last committed model/tool exchange. Cancellation,
network loss, application restart and an explicit budget stop preserve this state.

After reopening, reconcile durable state with the live registry. An absent live
process becomes Interrupted, not Running forever or Completed. Continue reuses
the same identity and saved work; do not infer the latest task by timestamp.
Restoring the parent tree must not automatically relaunch every child or replay
completed actions. A child continuation should report its result to the original
parent without requiring the parent to be currently running.

No UI can guarantee resumption in the middle of an arbitrary external effect.
Record started actions and known outcomes. If an interruption leaves an effect
uncertain, show that action for reconciliation before repeating it. A missing or
corrupt checkpoint is a visible recovery problem; do not pretend a fresh run is
the original continuation. Explicit Restart is separate from Continue.

## Settings and budgets

Ordinary saved runtime settings apply to existing conversations at the next
safe turn boundary. Already-running tools keep the configuration they started
with. Directory changes show the effective folder and wait for that boundary.
Never silently move artifacts or a running command to a new directory.

Prompt and tool-surface changes require an explicit application boundary that
writes a successor transcript generation, preserving the sealed history. Use
TinyAgents' prefix-refresh and tool-snapshot contracts; do not edit old transcripts
or silently restore revoked tools. Show when a setting cannot take effect yet.

Time and token budgets default to Off. The proposed time budget counts active
execution of this conversation across continuations, once for the parent tree
rather than summing parallel children; pausing freezes it. The shared token
ledger charges root and child model calls and survives restart. Reserve capacity
before parallel calls and reconcile provider usage afterward. The model receives
the configured scope, consumed/remaining budget and instructions to prioritize
completion. Stopping at a limit preserves a resumable state. Extending a limit
is an explicit operator action. Individual call/idle safeguards remain distinct
from an overall session timer and must be labeled accordingly.

## Authorizations

Requests identify the agent, action, target and requested scope. Offer only
decisions the backend supports, such as Allow once or a specific persistent rule.
Never imply that an arbitrary tool can be retried/skipped when no action exists.
Always-allowed entries support adding, removing, saving and visible failures.
Authentication failures and provider backoff are not approval requests.

## Implementation boundaries and current gaps

- React presents state with shared shadcn primitives, existing theme tokens,
  useT translations, keyboard navigation and user-scoped saved panel preferences.
- Core owns configuration, authorization, identity, persistence and scoped RPC.
  Do not route direct child messages through the main chat composer.
- TinyAgents owns generic inbox, run-queue mutation, checkpoint and resume
  semantics. Implement missing capability there and submit upstream before
  consuming an updated gitlink. No host-side replacement lifecycle.
- The existing child-question UI currently appends a parent user message.
  The command-center control RPC records ledger transitions; that alone does
  not prove recipient delivery or that execution actually resumed.
- Stable frontend activity ids, durable task/session ids and ledger run ids
  must be explicitly mapped. Never guess that they are interchangeable.
- The current run-queue edit/move API is a local upstream draft; UI integration
  and upstream availability remain work, not completed behavior.

## Completion evidence

Demonstrate addressed child messages during running/waiting/interrupted states;
delivery deduplication after reconnect; crash/reopen/continue with original task
identity and history; parent-child isolation; pending approvals; per-conversation
budgets and inherited child limits; configuration changes at their stated
boundary; grouping/archive/delete; and responsive, keyboard-accessible layouts.
Use controlled mock services for interruption and error scenarios. Visual mocks
are design references and never evidence that these runtime behaviors work.

## Left to the builder

Choose exact panel widths, icon sizes, responsive breakpoints and animation
timing using existing tokens. Keep collapse/resize preferences user scoped.
Prefer stable row lists with progressive disclosure over graphs, nested cards,
animated metrics or additional dashboards.
