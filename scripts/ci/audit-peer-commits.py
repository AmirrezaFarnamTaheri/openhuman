#!/usr/bin/env python3
"""Exhaustive offline Git history/branch/patch census across unrelated forks.

Read-only. Requires source and target Git repositories cloned locally (including
all remote branch refs). It does not apply patches or decide what is valuable.
Every source commit is retained in JSON, including reverted and merge commits.
"""
from __future__ import annotations

import argparse
import collections
import json
import pathlib
import re
import subprocess
import sys

DOMAIN_PATTERNS = {
    'security': r'secur|auth|permission|secret|cookie|policy|vault|sandbox|trust|injection|csrf|cors|credential',
    'providers': r'provider|model|inference|routing|gateway|key_pool|catalog|oauth|quota|failover|stream',
    'agents': r'agent|orchestrat|session|turn|tool|mcp|workflow|scheduler|context|runtime',
    'memory': r'memory|graph|vector|embedding|search|knowledge|rag|retrieve',
    'frontend': r'(^app/|frontend|interface|panel|widget|tsx?$|\.css$|i18n|locale|accessib)',
    'operations': r'(^\.github/|^scripts/|docker|deploy|ci|build|release|monitor|telemetry|observability)',
    'tests': r'(test|spec|fixture|benchmark|coverage|snapshot)',
    'docs': r'(^docs/|\.md$|readme|changelog)',
}
COMPILED = {key: re.compile(value, re.I) for key, value in DOMAIN_PATTERNS.items()}


def git(repo: pathlib.Path, *args: str, input_bytes: bytes | None = None) -> bytes:
    result = subprocess.run(
        ['git', '-C', str(repo), *args], input=input_bytes, stdout=subprocess.PIPE,
        stderr=subprocess.PIPE, check=False,
    )
    if result.returncode:
        raise RuntimeError(f"git {' '.join(args[:3])} failed: {result.stderr.decode(errors='replace')[:500]}")
    return result.stdout


def history(repo: pathlib.Path) -> list[str]:
    return git(repo, 'rev-list', '--topo-order', '--all').decode().splitlines()


def branches(repo: pathlib.Path) -> list[str]:
    return sorted(filter(None, git(repo, 'for-each-ref', '--format=%(refname:short)',
                                  'refs/heads/', 'refs/remotes/').decode().splitlines()))


def parents(repo: pathlib.Path, sha: str) -> list[str]:
    return git(repo, 'rev-list', '--parents', '-n', '1', sha).decode().split()[1:]


def metadata(repo: pathlib.Path, sha: str) -> tuple[str, str]:
    raw = git(repo, 'show', '-s', '--format=%aI%x00%s', sha).decode(errors='replace').strip()
    date, sep, subject = raw.partition('\0')
    if not sep:
        raise RuntimeError(f'git commit metadata malformed: {sha}')
    return date, subject.strip()


def changed_paths(repo: pathlib.Path, sha: str, parent_list: list[str]) -> list[str]:
    if parent_list:
        raw = git(repo, 'diff', '--no-ext-diff', '--name-only', '-z',
                  parent_list[0], sha, '--')
    else:
        raw = git(repo, 'diff-tree', '--root', '--no-commit-id', '--name-only', '-r',
                  '-z', sha)
    return sorted({x.decode(errors='surrogateescape') for x in raw.split(b'\0') if x})


def stable_patch_id(repo: pathlib.Path, sha: str, parent_list: list[str]) -> str | None:
    if parent_list:
        diff = git(repo, 'diff', '--no-ext-diff', '--binary', parent_list[0], sha)
    else:
        diff = git(repo, 'show', '--root', '--no-ext-diff', '--binary',
                   '--format=', sha)
    if not diff.strip():
        return None
    output = git(repo, 'patch-id', '--stable', input_bytes=diff)
    head = output.decode().split()
    return head[0] if head else None


def file_blob(repo: pathlib.Path, sha: str, path: str) -> str | None:
    # Use a literal pathspec-safe tree walk; do not eval refs from file contents.
    try:
        result = git(repo, 'rev-parse', '--verify', f'{sha}:{path}')
    except RuntimeError:
        return None
    return result.decode().strip()


def domains(paths: list[str], subject: str) -> list[str]:
    joined = ' '.join(paths) + ' ' + subject
    keys = [key for key, rgx in COMPILED.items() if rgx.search(joined)]
    return keys or ['misc']


def audit(source: pathlib.Path, target: pathlib.Path) -> dict:
    for repo in (source, target):
        if git(repo, 'rev-parse', '--is-shallow-repository').strip() == b'true':
            raise RuntimeError('Cannot claim exhaustive history from a shallow Git clone; fetch full history')
    source_ids = history(source)
    target_ids = history(target)
    target_set = set(target_ids)
    source_head = git(source, 'rev-parse', 'HEAD').decode().strip()
    target_head = git(target, 'rev-parse', 'HEAD').decode().strip()
    # Build the patch index even for unrelated DAGs; commit-SHA comparison alone
    # misses rebases/cherry-picks, and GitHub ahead/behind may return a 404.
    target_patches: set[str] = set()
    for sha in target_ids:
        pid = stable_patch_id(target, sha, parents(target, sha))
        if pid:
            target_patches.add(pid)
    records = []
    for sha in source_ids:
        par = parents(source, sha)
        date, subject = metadata(source, sha)
        paths = changed_paths(source, sha, par)
        pid = stable_patch_id(source, sha, par)
        if sha in target_set:
            status = 'identical-commit'
        elif pid and pid in target_patches:
            status = 'equivalent-patch'
        elif not paths:
            status = 'metadata-or-empty'
        else:
            status = 'candidate'
            # A deleted/replaced source change can still have valuable tests or
            # ideas. Keep the record; simply disclose that its code isn't current.
            if all(file_blob(source, source_head, p) != file_blob(source, sha, p)
                   for p in paths):
                status = 'not-current-in-source'
            elif all(file_blob(source, source_head, p) is not None and
                     file_blob(source, source_head, p) == file_blob(target, target_head, p)
                     for p in paths):
                status = 'same-tip-content'
        records.append({
            'sha': sha, 'date': date, 'subject': subject,
            'parents': par, 'is_merge': len(par) > 1,
            'patch_id': pid, 'paths': paths, 'file_count': len(paths),
            'domains': domains(paths, subject), 'status': status,
        })
    return {
        'schema': 1, 'status': 'COMPLETE',
        'source': str(source), 'target': str(target),
        'source_head': source_head, 'target_head': target_head,
        'source_commits': len(source_ids), 'target_commits': len(target_ids),
        'source_branches': branches(source), 'target_branches': branches(target),
        'status_counts': dict(collections.Counter(r['status'] for r in records)),
        'domain_counts': dict(collections.Counter(d for r in records for d in r['domains'])),
        'commits': records,
    }


def markdown(result: dict) -> str:
    lines = [
        '# Peer Git commit convergence census', '',
        f"Result: **{result['status']}** (complete commit inventory; **not** a semantic assessment)",
        f"Source commits: **{result['source_commits']}**; target commits: **{result['target_commits']}**.",
        f"Source refs: {len(result['source_branches'])}; target refs: {len(result['target_branches'])}.",
        'History may be unrelated; stable patch IDs are compared even without a Git merge-base.',
        'Reverted and merge commits remain in the JSON even if they appear redundant.',
        '', '## Status census', '',
    ]
    for status, n in sorted(result['status_counts'].items()):
        lines.append(f'- {status}: {n:,}')
    lines.extend(['', '## Domains (overlap permitted)', ''])
    for domain, n in sorted(result['domain_counts'].items(), key=lambda x: -x[1]):
        lines.append(f'- {domain}: {n:,}')
    lines += ['', '## Review candidates (first 150; full list in JSON)', '',
              '| Commit | Subject | Domains | Files | Status |', '|---|---|---|---:|---|']
    candidates = [r for r in result['commits'] if r['status'] not in
                  ('identical-commit', 'equivalent-patch', 'metadata-or-empty')]
    for r in candidates[:150]:
        sub = r['subject'].replace('|','/').replace('\n',' ')
        lines.append(f"| `{r['sha'][:12]}` | {sub[:160]} | {', '.join(r['domains'])} | "
                     f"{r['file_count']} | {r['status']} |")
    lines += ['', f'Review candidates in JSON (including reverted changes): {len(candidates)}.',
              'Do not cherry-pick anything solely because it is classified as a candidate.', '']
    return '\n'.join(lines)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', required=True, type=pathlib.Path)
    parser.add_argument('--target', required=True, type=pathlib.Path)
    parser.add_argument('--output', required=True, type=pathlib.Path)
    parser.add_argument('--summary', required=True, type=pathlib.Path)
    args = parser.parse_args(argv)
    for p in (args.source, args.target):
        git(p, 'rev-parse', '--is-inside-work-tree')
    report = audit(args.source, args.target)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.summary.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    args.summary.write_text(markdown(report), encoding='utf-8')
    print(f"source_commits={report['source_commits']} target_commits={report['target_commits']} "
          f"statuses={report['status_counts']}")
    return 0


if __name__ == '__main__':
    sys.exit(main())
