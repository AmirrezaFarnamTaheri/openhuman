#!/usr/bin/env python3
"""Resolve reviewed upstream merge overlaps at individual hunk granularity.

One-off sync helper: Git's ort merge has already merged all non-overlapping
changes; this script never replaces an entire conflicted file. It fails closed
when upstream changes the expected conflict shape. Remove after the sync.
"""
from pathlib import Path
import re
import subprocess
import sys

def run(*args):
    return subprocess.check_output(args, text=True).strip()

CONFLICT = re.compile(
    r"(?ms)^<<<<<<< HEAD\n(?P<ours>.*?)^\\|\\|\\|\\|\\|\\|\\|[^\n]*\n"
    r"(?P<base>.*?)^=======\n(?P<theirs>.*?)^>>>>>>>[^\n]*\n"
)
# Choices correspond to the October 27 recorded zdiff3 conflict inventory.
# All nonconflicting upstream and local edits have already been kept by ort.
# ours = preserve tested fork fix; theirs = accept newer upstream contract.
POLICY = {
    "Cargo.lock": ["ours"] * 7,
    "Cargo.toml": ["ours"],
    "app/package.json": ["ours"],
    "crates/openhuman-app/Cargo.lock": ["ours"] * 5 + ["theirs", "theirs"],
    "crates/openhuman-app/Cargo.toml": ["ours"],
    "crates/openhuman-app/tauri.conf.json": ["ours"],
    "crates/openhuman-cli/Cargo.toml": ["theirs"],
    "crates/openhuman-core/src/agent/orchestration/tools/tools_e2e_thread_delete_tests.rs": ["theirs", "theirs"],
    "crates/openhuman-core/src/channels/routes_tests.rs": ["ours"],
    "crates/openhuman-core/src/channels/runtime/mod.rs": ["exports"],
    "crates/openhuman-core/src/channels/runtime/startup/start_channels.rs": ["theirs"],
    "crates/openhuman-core/src/channels/runtime/startup/turn_parts.rs": ["config"],
    "crates/openhuman-core/src/channels/tests/runtime_tool_calls.rs": ["context"],
    "crates/openhuman-core/src/core/cli.rs": ["dispatch"],
    "crates/openhuman-core/src/cron/client.rs": ["ours"],
    "crates/openhuman-core/src/doctor/checks/config_semantics_tests.rs": ["theirs"],
    "crates/openhuman-core/src/doctor/checks/workspace_tests.rs": ["theirs"],
    "crates/openhuman-core/src/mcp/server/tools_tests_dispatch_tests.rs": ["ours", "ours"],
    "crates/openhuman-core/src/storage/blob_sweep_tests.rs": ["theirs"],
    "crates/openhuman-core/src/web_chat/README.md": ["ours"],
    "crates/openhuman-core/src/web_chat/turn_guards/backstop.rs": ["ours"],
    "crates/openhuman-core/src/web_chat/turn_guards/backstop_tests.rs": ["ours", "ours"],
    "crates/openhuman-embed/examples/mcp.rs": ["theirs", "ours", "ours"],
    "crates/openhuman-embed/examples/support/mod.rs": ["ours"],
    "docs/gitbooks/en/developing/architecture/agent-harness.md": ["theirs", "ours"],
    "scripts/__tests__/self-hosted-lanes.test.mjs": ["ours"],
    "scripts/ci/agent-runtime-boundary-baseline.json": ["ours"] * 6,
    "scripts/ci/check-dep-sim-calibration.sh": ["theirs"],
    "scripts/ci/check-gated-test-allowlist.sh": ["ours"],
    "scripts/ci/check-storage-schema.mjs": ["ours", "ours"],
    "scripts/ci/process-statics.json": ["ours"] * 4,
    "scripts/ci/saas-ambient-baseline.json": ["ours"] * 10,
    "scripts/ci/self-hosted/lanes-plan.mjs": ["ours"] * 3,
    "scripts/kernel-floor.limits": ["theirs"],
    "tests/agent_harness_e2e.rs": ["ours"] * 13,
}
# These are semantically composed: keep both the fork's new capabilities and
# the upstream module runtime's new exports and model source contract.
CUSTOM = {
    "exports": (
        "pub use startup::{start_channels, start_channels_in};\n"
        "#[cfg(feature = \"channels\")]\n"
        "pub(crate) use startup::ChannelHandle;\n"
        "#[cfg(all(test, feature = \"channels\"))]\n"
        "pub(crate) use startup::TestChannel;\n"
        "#[cfg(all(test, feature = \"channels\"))]\n"
        "pub(crate) use startup::{prepare_module_runtime_for_test, ModuleRuntimeTestGuard};\n"
    ),
    "config": (
        "        temperature: inference_config.default_temperature,\n"
        "        max_tool_iterations: inference_config.agent.max_tool_iterations,\n"
    ),
    "context": (
        "    let mut config = crate::config::Config::default();\n"
        "    config.cloud_providers.push(crate::config::schema::CloudProviderCreds {\n"
        "        id: \"p_managed_models_test\".to_string(),\n"
        "        slug: \"openhuman\".to_string(),\n"
        "        label: \"Managed Backend\".to_string(),\n"
        "        endpoint: \"https://api.openhuman.ai/v1\".to_string(),\n"
        "        auth_style: crate::config::schema::AuthStyle::OpenhumanJwt,\n"
        "        default_model: Some(\"default-model\".to_string()),\n"
        "        ..Default::default()\n"
        "    });\n"
        "    let mut runtime_ctx = ChannelRuntimeContext {\n"
    ),
    "dispatch": (
        "            let dispatch = if headless_agent_chat {\n"
        "                let config = crate::config::ops::load_config_with_timeout()\n"
    ),
}
# These were moved/renamed in upstream. Do not reintroduce orphaned files.
UPSTREAM_DELETIONS = {
    "crates/openhuman-core/src/web_chat/ops/turn_guards.rs",
    "crates/openhuman-core/src/web_chat/ops/turn_guards_tests.rs",
    "tests/inbound_linked_e2e.rs",
}
SUBMODULE = "vendor/tinyagents"
FORK_SUBMODULE_SHA = "ca5c45f8c0e2136098df776ea2f964bd1345aecf"
raw = subprocess.check_output(["git", "diff", "--name-only", "--diff-filter=U", "-z"])
paths = [p.decode() for p in raw.split(b"\0") if p]
known = set(POLICY) | UPSTREAM_DELETIONS | {SUBMODULE}
unexpected = sorted(set(paths) - known)
if unexpected:
    raise SystemExit(f"Unreviewed merge paths: {unexpected}")
counts = {}
for path in paths:
    if path in UPSTREAM_DELETIONS:
        subprocess.check_call(["git", "rm", "-f", "--", path])
        print("Accept upstream move/delete:", path)
        continue
    if path == SUBMODULE:
        # Preserve both logical submodule histories later, after the independently
        # reviewable TinyAgents fork merge. This temporary pin does NOT finish
        # submodule convergence; it is intentionally reported as a blocker.
        subprocess.check_call([
            "git", "update-index", "--add", "--cacheinfo",
            f"160000,{FORK_SUBMODULE_SHA},{SUBMODULE}"
        ])
        print("TEMPORARY fork submodule pin; must replace with merged TinyAgents SHA")
        continue
    text = Path(path).read_text()
    selections = POLICY[path]
    seen = 0
    def resolve(m):
        global seen
        if seen >= len(selections):
            raise RuntimeError(f"Unexpected extra hunk in {path}")
        choice = selections[seen]
        seen += 1
        return m.group(choice) if choice in ("ours", "theirs") else CUSTOM[choice]
    resolved = CONFLICT.sub(resolve, text)
    if seen != len(selections):
        raise RuntimeError(f"{path}: expected {len(selections)} merge hunks, found {seen}")
    if any(mark in resolved for mark in ("<<<<<<< HEAD\n", ">>>>>>> e53cc0c")):
        raise RuntimeError(f"{path}: unresolved conflict marker remains")
    Path(path).write_text(resolved)
    subprocess.check_call(["git", "add", "--", path])
    counts[path] = seen
remaining = run("git", "diff", "--name-only", "--diff-filter=U")
if remaining:
    raise SystemExit(f"Unresolved paths remain: {remaining}")
print(f"Resolved {sum(counts.values())} content hunks and {len(UPSTREAM_DELETIONS)} moves; submodule needs follow-up.")
