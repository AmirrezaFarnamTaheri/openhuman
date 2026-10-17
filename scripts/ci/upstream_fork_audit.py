#!/usr/bin/env python3
"""Exhaustively inventory upstream forks and compare default-branch commits.

Run with GITHUB_TOKEN set. The script is read-only, checkpointed, and
revision-pinned. A partial audit returns exit status 2 rather than asserting
that every upstream fork was checked.
"""
from __future__ import annotations

import argparse
import datetime as dt
import json
import os
import time
from collections import deque
from pathlib import Path
from typing import Any
from urllib.error import HTTPError, URLError
from urllib.parse import quote
from urllib.request import Request, urlopen


class RequestBudgetExhausted(RuntimeError):
    """Request budget or GitHub rate limit exhausted."""


class GitHubApi:
    def __init__(self, token: str, max_requests: int) -> None:
        if not token:
            raise ValueError("GITHUB_TOKEN is required for a full fork census")
        self.token = token
        self.max_requests = max_requests
        self.requests = 0

    def get(self, path: str) -> Any:
        if not path.startswith("/repos/") or "/../" in path or path.endswith("/.."):
            raise ValueError(f"invalid GitHub API path: {path}")
        request = Request(
            "https://api.github.com" + path,
            headers={
                "Authorization": f"Bearer {self.token}",
                "Accept": "application/vnd.github+json",
                "X-GitHub-Api-Version": "2022-11-28",
                "User-Agent": "openhuman-upstream-fork-audit/1.0",
            },
        )
        # GitHub occasionally disconnects a long-running fork comparison
        # before it returns headers. Bound retries and count every attempt
        # against the scan budget; if still unavailable, the caller saves an
        # INCOMPLETE checkpoint and the next run resumes from it.
        for attempt in range(3):
            if self.max_requests and self.requests >= self.max_requests:
                raise RequestBudgetExhausted(
                    f"request budget ({self.max_requests}) exceeded"
                )
            self.requests += 1
            try:
                with urlopen(request, timeout=35) as response:
                    return json.load(response)
            except HTTPError as exc:
                if exc.code in {403, 429} and (
                    exc.headers.get("x-ratelimit-remaining") == "0"
                    or exc.headers.get("retry-after")
                ):
                    raise RequestBudgetExhausted(
                        f"GitHub API rate limit after {self.requests} requests"
                    ) from exc
                if exc.code not in {500, 502, 503, 504}:
                    raise RuntimeError(f"GitHub HTTP {exc.code}: {path}") from exc
                problem = f"GitHub HTTP {exc.code}: {path}"
            except (URLError, OSError) as exc:
                # http.client.RemoteDisconnected is a ConnectionResetError,
                # not a URLError, and was escaping as an uncaught traceback.
                problem = f"GitHub API transport failure for {path}: {exc}"
            if attempt == 2:
                raise RequestBudgetExhausted(
                    f"GitHub temporarily unavailable after {self.requests} requests: {problem}"
                )
            time.sleep(2 ** attempt)


def enumerate_forks(api: Any, upstream: str) -> dict[str, dict[str, Any]]:
    """Page through all visible forks, traversing nested forks too."""
    queue: deque[str] = deque([upstream])
    visited: set[str] = set()
    seen_ids: set[int] = set()
    forks: dict[str, dict[str, Any]] = {}
    while queue:
        parent = queue.popleft()
        if parent in visited:
            continue
        visited.add(parent)
        page = 1
        while True:
            path = f"/repos/{parent}/forks?per_page=100&page={page}&sort=oldest"
            batch = api.get(path)
            if not isinstance(batch, list):
                raise ValueError(f"unexpected fork-list response: {path}")
            for row in batch:
                name, repo_id = row.get("full_name"), row.get("id")
                if not isinstance(name, str) or name.count("/") != 1:
                    raise ValueError(f"invalid fork full_name from {path}: {name!r}")
                if not isinstance(repo_id, int):
                    raise ValueError(f"missing fork id in {path}: {name}")
                if repo_id in seen_ids or name == upstream:
                    continue
                seen_ids.add(repo_id)
                forks[name] = {
                    "id": repo_id,
                    "full_name": name,
                    "default_branch": row.get("default_branch") or "",
                    "stargazers_count": row.get("stargazers_count", 0),
                    "pushed_at": row.get("pushed_at"),
                    "archived": bool(row.get("archived")),
                    "disabled": bool(row.get("disabled")),
                }
                if row.get("forks_count", 0) > 0:
                    queue.append(name)
            if len(batch) < 100:
                break
            page += 1
    return forks


def compare_forks(
    api: Any,
    upstream: str,
    upstream_sha: str,
    rows: list[dict[str, Any]],
    previous: dict[str, dict[str, Any]],
) -> dict[str, dict[str, Any]]:
    """Compare each fork against the pinned upstream SHA, retaining progress."""
    results = dict(previous)
    for row in rows:
        name = row["full_name"]
        if name in results:
            continue
        owner = name.split("/", 1)[0]
        branch = row.get("default_branch") or ""
        if not branch:
            raise ValueError(f"fork has no default branch: {name}")
        head = quote(f"{owner}:{branch}", safe=":")
        data = api.get(f"/repos/{upstream}/compare/{upstream_sha}...{head}")
        if not isinstance(data.get("ahead_by"), int):
            raise ValueError(f"invalid ahead_by in comparison for {name}")
        results[name] = {
            **row,
            "ahead_by": data["ahead_by"],
            "behind_by": data.get("behind_by", 0),
            "status": data.get("status", "unknown"),
            "merge_base_sha": (data.get("merge_base_commit") or {}).get("sha"),
            "recent_unique_commits": [
                {
                    "sha": commit.get("sha"),
                    "message": (commit.get("commit") or {}).get("message", "").split("\n", 1)[0][:200],
                }
                for commit in (data.get("commits") or [])[-8:]
            ],
        }
    return results


def ahead_rows(comparisons: dict[str, dict[str, Any]]) -> list[dict[str, Any]]:
    return sorted(
        (row for row in comparisons.values() if row.get("ahead_by", 0) > 0),
        key=lambda row: (
            -row["ahead_by"], -row.get("stargazers_count", 0), row["full_name"]
        ),
    )


def markdown_report(
    upstream: str,
    sha: str,
    enumerated: int,
    comparisons: dict[str, dict[str, Any]],
    errors: dict[str, str],
) -> str:
    complete = len(comparisons) == enumerated and not errors
    rows = ahead_rows(comparisons)
    lines = [
        "# Upstream fork-ahead census",
        "",
        f"- Upstream: {upstream} at {sha}",
        f"- Result: {'COMPLETE' if complete else 'INCOMPLETE'}",
        f"- Forks discovered: {enumerated}",
        f"- Forks successfully compared: {len(comparisons)}",
        f"- Forks ahead on their default branches: {len(rows)}",
        f"- Unresolved comparison errors: {len(errors)}",
        "",
        "This report is pinned to one upstream revision. A fork with an older",
        "default branch can still carry an ahead feature branch or pull request.",
        "Inspect each unique patch before integration; do not mass-merge fork heads.",
        "",
        "| Fork | Ahead | Behind | Stars | Last unique commit |",
        "|---|---:|---:|---:|---|",
    ]
    for row in rows:
        name = row["full_name"]
        commits = row.get("recent_unique_commits") or []
        summary = (commits[-1].get("message", "") if commits else "").replace("|", "/")
        lines.append(
            f"| [{name}](https://github.com/{name}) | {row['ahead_by']} | "
            f"{row.get('behind_by', 0)} | {row.get('stargazers_count', 0)} | {summary} |"
        )
    if errors:
        lines += ["", "## Unresolved forks", ""]
        for name, reason in sorted(errors.items()):
            lines.append(f"- {name}: {reason}")
    return "\n".join(lines) + "\n"


def write_checkpoint(path: Path, state: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    tmp = path.with_name(path.name + ".tmp")
    tmp.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    tmp.replace(path)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--upstream", default="tinyhumansai/openhuman")
    parser.add_argument("--upstream-sha", default="")
    parser.add_argument("--budget", type=int, default=800)
    parser.add_argument("--checkpoint", type=Path, default=Path(".fork-audit/checkpoint.json"))
    parser.add_argument("--output-json", type=Path, default=Path(".fork-audit/forks.json"))
    parser.add_argument("--output-md", type=Path, default=Path(".fork-audit/FORK_AHEAD_REPORT.md"))
    args = parser.parse_args()
    if args.upstream.count("/") != 1 or args.budget < 0:
        parser.error("invalid upstream or request budget")
    api = GitHubApi(os.getenv("GITHUB_TOKEN", ""), args.budget)
    if args.checkpoint.exists():
        state = json.loads(args.checkpoint.read_text(encoding="utf-8"))
        if state.get("upstream") != args.upstream:
            parser.error("checkpoint belongs to another upstream")
        if args.upstream_sha and state["sha"] != args.upstream_sha:
            parser.error("checkpoint pinned to another upstream SHA")
    else:
        sha = args.upstream_sha
        metadata = api.get(f"/repos/{args.upstream}")
        if not sha:
            branch = metadata.get("default_branch") or "main"
            sha = api.get(f"/repos/{args.upstream}/commits/{quote(branch, safe='')}")["sha"]
        state = {
            "upstream": args.upstream, "sha": sha,
            "declared_fork_count": metadata.get("forks_count", 0),
            "forks": {}, "comparisons": {}, "errors": {},
        }
    try:
        if not state["forks"]:
            state["forks"] = enumerate_forks(api, args.upstream)
            write_checkpoint(args.checkpoint, state)
        for name, row in sorted(state["forks"].items()):
            if name in state["comparisons"]:
                continue
            try:
                state["comparisons"].update(
                    compare_forks(api, args.upstream, state["sha"], [row], {})
                )
                state["errors"].pop(name, None)
            except RequestBudgetExhausted:
                raise
            except (RuntimeError, ValueError, KeyError) as exc:
                state["errors"][name] = str(exc)
            if len(state["comparisons"]) % 25 == 0:
                write_checkpoint(args.checkpoint, state)
        if len(state["forks"]) < state.get("declared_fork_count", 0):
            state["errors"]["_visibility_gap"] = (
                f"GitHub reports {state['declared_fork_count']} forks but only "
                f"{len(state['forks'])} were visible in the API"
            )
    except RequestBudgetExhausted as exc:
        print(f"Audit paused: {exc}")
    finally:
        write_checkpoint(args.checkpoint, state)
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(json.dumps(state, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        args.output_md.parent.mkdir(parents=True, exist_ok=True)
        args.output_md.write_text(
            markdown_report(
                args.upstream, state["sha"], len(state["forks"]),
                state["comparisons"], state["errors"]
            ),
            encoding="utf-8",
        )
    complete = (
        bool(state["forks"]) and
        len(state["comparisons"]) == len(state["forks"]) and
        not state["errors"]
    )
    print(
        f"forks={len(state['forks'])} compared={len(state['comparisons'])} "
        f"ahead={len(ahead_rows(state['comparisons']))} "
        f"errors={len(state['errors'])} complete={complete} requests={api.requests}"
    )
    return 0 if complete else 2


if __name__ == "__main__":
    raise SystemExit(main())
