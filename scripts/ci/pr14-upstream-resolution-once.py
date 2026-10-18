#!/usr/bin/env python3
"""One-time, source-aware resolution for the October 2026 canonical OpenHuman integration.

Requires an unfinished three-way git merge. Never auto-resolve an unknown path.
Remove this script after the merge. This is not a generic conflict policy.
"""
import json
import os
from pathlib import Path
import re
import subprocess
import sys

def git(*args):
    return subprocess.check_output(["git", *args], text=True)

def stage(which, file):
    return git("show", f":{which}:{file}")

def put(file, text):
    Path(file).parent.mkdir(parents=True, exist_ok=True)
    Path(file).write_text(text)

def resolve_hunks(content, policy):
    lines = content.splitlines(keepends=True)
    out = []
    at = 0
    resolved = 0
    while at < len(lines):
        if not lines[at].startswith("<<<<<<< "):
            out.append(lines[at])
            at += 1
            continue
        resolved += 1
        at += 1
        a, b = [], []
        while at < len(lines) and not lines[at].startswith("======="):
            a.append(lines[at]); at += 1
        if at == len(lines):
            raise RuntimeError("unterminated ours conflict")
        at += 1
        while at < len(lines) and not lines[at].startswith(">>>>>>> "):
            b.append(lines[at]); at += 1
        if at == len(lines):
            raise RuntimeError("unterminated theirs conflict")
        at += 1
        if policy == "ours":
            out.extend(a)
        elif policy == "theirs":
            out.extend(b)
        elif policy == "both":
            out.extend(a + b)
        else:
            raise ValueError(policy)
    if resolved < 1:
        raise RuntimeError(f"requested hunk resolution {policy} but found no markers")
    return "".join(out), resolved

PREFIX = "crates/openhuman-core/src/"
RULES = {
    PREFIX+"agent/multimodal.rs": "theirs", # newer workspace-scoped attachment store
    PREFIX+"agent/orchestration/background_completions/thread_index.rs": "ours",
    PREFIX+"agent/orchestration/mod.rs": "ours", # retain fork coding_handoff + command_center
    PREFIX+"agent/subagent_host/lifecycle.rs": "theirs", # detached lineage + typed terminal state
    PREFIX+"agent/subagent_host/ops/runner.rs": "ours", # fork tier module + artifact offload imports
    PREFIX+"modules/storage_import_tinyjuice_savings.rs": "ours", # keep explicit dispatch context
    PREFIX+"tools/impl/system/shell.rs": "ours",
    PREFIX+"triage/decision_tests.rs": "both",
    "docs/gitbooks/en/developing/architecture/agent-harness.md": "theirs",
    "scripts/ci/check-openhuman-rust-layout.mjs": "ours",
    "scripts/ci/storage-bypass-debt.mjs": "theirs",
}
SPECIAL = {
    PREFIX+"agent/tinyagents/middleware/tool_policy.rs",
    "scripts/ci/process-statics.json",
}
RELOCATED = {
    PREFIX+"agent_runs/replay/local_checks.rs",
    PREFIX+"agent_runs/replay/local_checks_tests.rs",
    PREFIX+"triage/evaluator/prompt_tests.rs",
}
KEEP_DELETED_BY_UPSTREAM = {
    PREFIX+"agent/orchestration/tools/spawn_async_subagent_execute.rs",
    PREFIX+"agent/tinyagents/use_skill_dispatch.rs",
}
GITLINK = "vendor/tinyagents"
EXPECTED = set(RULES) | SPECIAL | RELOCATED | KEEP_DELETED_BY_UPSTREAM | {GITLINK}

conflicts = set(git("diff", "--name-only", "--diff-filter=U").splitlines())
unknown = conflicts - EXPECTED
missing = EXPECTED - conflicts
if unknown:
    raise SystemExit("Unreviewed merge conflicts; refusing: " + repr(sorted(unknown)))
if missing:
    print("Previously conflicting paths now auto-merge:", sorted(missing))

for file, policy in RULES.items():
    if file not in conflicts:
        continue
    text, count = resolve_hunks(Path(file).read_text(), policy)
    if file.endswith("/thread_index.rs"):
        text = text.replace(
            "use super::{router_for_workspace, state, HostState, SESSION_THREADS_CAP};",
            "use super::{state, HostState, SESSION_THREADS_CAP};",
        ).replace(
            "Some(router_for_workspace(&workspace))",
            "Some(super::router_for_workspace_ctx(ctx, &workspace))",
        )
    if file.endswith("/triage/decision_tests.rs"):
        # The shared attribute before the conflict covers the first fork test.
        # The canonical regression appended after it needs its own #[test].
        text = text.replace(
            "fn owner_relaxed_json_recovery_preserves_action_normalization()",
            "#[test]\\nfn owner_relaxed_json_recovery_preserves_action_normalization()",
            1,
        )
    if file.endswith("/system/shell.rs"):
        original = "use super::shell_support::{append_verification_receipt, exec_result, python_utf8_env};"
        expanded = (
            "use super::shell_support::{append_verification_receipt, command_failure, "
            "exec_result, python_utf8_env, tool_timeout_message};"
        )
        if original not in text:
            raise SystemExit("Unexpected shell imports; not auto-resolving")
        text = text.replace(original, expanded, 1)
    if file == "scripts/ci/storage-bypass-debt.mjs":
        existing = 'const base = git(["merge-base", ref, "HEAD"]).trim();'
        if existing not in text:
            raise SystemExit("Unexpected upstream debt base resolution")
        text = text.replace(existing, 'const base = canonicalMergeBase(git);', 1)
    put(file, text)
    print(f"resolved {file}: {count} explicitly reviewed hunk(s) ({policy})")

if PREFIX+"agent/tinyagents/middleware/tool_policy.rs" in conflicts:
    file = PREFIX+"agent/tinyagents/middleware/tool_policy.rs"
    ours, theirs = stage(2, file), stage(3, file)
    a = ours.index("/// The borrowed session-scoped view")
    b = ours.index("#[async_trait]\nimpl ToolCallPolicy", a)
    view_decl = ours[a:b]
    a = ours.index("/// Finds the registered [`Tool`]")
    b = ours.index("#[async_trait]\nimpl ToolMiddleware", a)
    view_impl = ours[a:b]
    insert_before = "/// The session-less allow-all gate"
    insert_impl_before = "struct SessionCeilingPolicy {"
    if not all(needle in theirs for needle in (insert_before, insert_impl_before)):
        raise SystemExit("Upstream tool policy architecture changed again")
    merged = theirs.replace(insert_before, view_decl + insert_before, 1)
    merged = merged.replace(insert_impl_before, view_impl + insert_impl_before, 1)
    put(file, merged)
    print(f"resolved {file}: canonical gate + fork session permission view")

if "scripts/ci/process-statics.json" in conflicts:
    file = "scripts/ci/process-statics.json"
    a, b = json.loads(stage(2, file)), json.loads(stage(3, file))
    indexed = {}
    for entry in a + b:
        sig = tuple(entry.get(k, "") for k in ("file", "scope", "name", "type"))
        indexed[sig] = entry
    # Keep both sets of audited ownership evidence, favor canonical annotations
    # for identical declarations. Drop only records for deleted source paths.
    records = [e for e in indexed.values() if Path(e["file"]).is_file()]
    records.sort(key=lambda e: (e["file"], e.get("scope", ""), e["name"], e["type"]))
    put(file, json.dumps(records, indent=2, ensure_ascii=False) + "\n")
    print(f"resolved {file}: {len(records)} exact owner records")

for file in sorted(RELOCATED & conflicts):
    if not Path(file).exists():
        raise SystemExit(f"Git did not relocate {file}; manual resolution required")
    print(f"preserving fork tests at canonical moved location {file}")

for file in sorted(KEEP_DELETED_BY_UPSTREAM & conflicts):
    if not Path(file).is_file():
        raise SystemExit(f"Fork implementation vanished during upstream delete: {file}")
    print(f"retaining modified fork implementation {file} until adapter tests verify relocation")

if GITLINK in conflicts:
    sha = os.environ.get("OPENHUMAN_TINYAGENTS_CONVERGED_SHA", "69eb675a64ab0317531ce37a9a117c0e09dc42cc")
    if not re.fullmatch(r"[a-f0-9]{40}", sha):
        raise SystemExit("Invalid TinyAgents dependency commit")
    subprocess.check_call(["git", "update-index", "--add", "--cacheinfo", "160000", sha, GITLINK])
    print(f"TinyAgents fork gitlink: {sha}")

subprocess.check_call(["git", "add", "-A"])
remaining = git("diff", "--name-only", "--diff-filter=U").strip()
if remaining:
    raise SystemExit("Unresolved index stages remain:\n" + remaining)
for file in SPECIAL | set(RULES):
    if Path(file).is_file() and re.search(r"^(<<<<<<< |=======|>>>>>>> )", Path(file).read_text(), re.M):
        raise SystemExit("Conflict markers remain in " + file)
print(f"Resolved {len(conflicts)} reviewed conflict path(s).")
