# Native local-check receipts — Canny / SWE-agent / Open SWE convergence

Status: implemented on draft PR #8, **not compilation-tested** on the current head.

## Ownership and data flow

The existing OpenHuman ShellTool remains the only OS-command authority. After host policy/sandbox execution, it classifies a narrow set of simple test, build, type-check and lint commands and attaches a versioned host-only tinytools ToolResult.metadata value under `openhuman_local_check_v1`. The same TinyAgents ToolCompleted event and durable journal already used for run replay persist that value. The normal tool output delivered to the model does not include the metadata. No second agent runtime, command log, or credential store is created.

The evidence records only `recognized`, `passed`, `status`, `exit_status` (zero or unknown), `scope=local_check_only`, and `task_verified=false`. It contains no shell command, arguments, stdout, stderr, environment variables, key material or even a reversible command fingerprint. Shell denial cannot create a positive receipt. A failed/unknown command never counts as successful.

To keep observed exit codes meaningful, shell wrappers are **not** treated as verification when a pipeline, redirection, command chaining, shell expansion, comments, help/list/collect-only/no-run/dry-run options, or other opaque shell syntax is present. Many legitimate complex commands will therefore receive no receipt; that is safer than claiming a test passed when the outer process exit hides a failure. The recognized command subset includes simple Cargo, Go, Python, Node and npm/pnpm test/build/lint forms.

## Read-only replay API

`openhuman.agent_run_local_checks({"run_id":"..."})` reads the existing scoped durable journal, at most 12 pages of 256 events. It returns a sanitized summary:

- `status`: `local_check_observed`, `stale_local_check`, `failed_local_check`, `no_local_check`, or `partial_journal`
- `passed_checks`, `failed_checks`, `last_check_offset`, `last_possible_mutation_offset`
- `unclassified_shell_calls`, `check_after_last_known_mutation`, `journal_complete`
- `task_verified=false`, `scope=observed_journal_local_checks_only`

Direct file-edit tools and other potentially mutating workflows invalidate earlier checks. An unclassified shell may mutate code, so it also invalidates freshness. A missing/evicted event or replay-page limit yields `partial_journal`. The summary does not echo event payloads or secrets. It is scoped to the caller's configured workspace through the same RPC resolver as run events/status. This surface adds **observability**, not additional execution privileges.

## Evidence hierarchy and remaining acceptance criteria

1. A successful exit recorded through ShellTool is a *local process outcome*, not verified requirements coverage. An ordinary successful shell command is not automatically a test.
2. A journal with a check after every known source mutation supplies a limited freshness fact. It does **not** prove that shell commands or external agents made no unobserved changes.
3. A machine-verifiable completion receipt additionally needs a source tree/generation hash tied to the check, a manifest of tests actually run, a complete effects journal, and GitHub CI check-run results against the **final exact commit SHA**. Neither the model's final answer nor a local status marked `passed` substitutes for that.
4. The implementation is not ready for release until `cargo test -p openhuman shell_verification`, `cargo test -p openhuman local_checks`, `cargo check -p openhuman`, platform builds, permission-adversarial tests, and a latest-head CI run complete.

The pinned TinyAgents observability contract provides typed `ToolStarted`, `ToolCompleted`, `ToolFailed` events and a host-only metadata channel. This implementation adapts Canny's post-edit freshness insight to that existing seam without importing Canny's independent hook process or model judge.

## Manual CI entry points if connector-created commits do not trigger GitHub Actions

The repository's existing Test and CI Full workflows support `workflow_dispatch`. An operator with Actions permission can trigger them on the PR branch:

```bash
gh workflow run test.yml -R AmirrezaFarnamTaheri/openhuman --ref feat/peer-provider-convergence
gh workflow run ci-full.yml -R AmirrezaFarnamTaheri/openhuman --ref feat/peer-provider-convergence
```

These commands are **not** recorded as having been executed. After a run, check that its `head_sha` is the final PR head and inspect failing jobs before marking the PR ready to merge. Do not use old commits' green checks.
