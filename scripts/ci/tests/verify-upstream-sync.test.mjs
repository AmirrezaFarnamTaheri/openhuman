import test from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import { mkdtempSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const script = resolve(import.meta.dirname, '../verify-upstream-sync.mjs');

function git(cwd, ...args) {
  return execFileSync('git', ['-C', cwd, ...args], { encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] }).trim();
}

function commit(cwd, name, content) {
  writeFileSync(join(cwd, name), content);
  git(cwd, 'add', name);
  git(cwd, 'commit', '-m', name);
  return git(cwd, 'rev-parse', 'HEAD');
}

function fixture(fn) {
  const cwd = mkdtempSync(join(tmpdir(), 'upstream-sync-'));
  try {
    git(cwd, 'init', '-b', 'main');
    git(cwd, 'config', 'user.name', 'Test');
    git(cwd, 'config', 'user.email', 'test@example.com');
    commit(cwd, 'base.txt', 'base');
    git(cwd, 'branch', 'fork');
    git(cwd, 'branch', 'upstream');
    git(cwd, 'switch', 'fork');
    const originalFork = commit(cwd, 'fork.txt', 'important fork behavior');
    git(cwd, 'switch', 'upstream');
    const latestUpstream = commit(cwd, 'upstream.txt', 'new upstream behavior');
    return fn({ cwd, originalFork, latestUpstream });
  } finally {
    rmSync(cwd, { recursive: true, force: true });
  }
}

function run({ cwd, originalFork, expected = 1, candidate = 'HEAD' }) {
  return spawnSync(process.execPath, [script,
    '--original-fork', originalFork, '--upstream', 'upstream',
    '--candidate', candidate, '--expected-original-unique', String(expected),
  ], { cwd, encoding: 'utf8' });
}

test('rejects candidate that omits the original fork commit', () => fixture(input => {
  git(input.cwd, 'switch', 'upstream');
  const result = run(input);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /original fork.*ancestor/i);
}));

test('rejects candidate that is still behind latest upstream', () => fixture(input => {
  git(input.cwd, 'switch', 'fork');
  const result = run(input);
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /upstream.*ancestor/i);
}));

test('accepts real merge ancestry without rewriting either history', () => fixture(input => {
  git(input.cwd, 'switch', 'fork');
  git(input.cwd, 'merge', '--no-ff', '-m', 'integrate latest upstream', 'upstream');
  const candidate = git(input.cwd, 'rev-parse', 'HEAD');
  const result = run(input);
  assert.equal(result.status, 0, result.stderr);
  const report = JSON.parse(result.stdout);
  assert.equal(report.candidate, candidate);
  assert.equal(report.originalFork, input.originalFork);
  assert.equal(report.latestUpstream, input.latestUpstream);
  assert.equal(report.behind, 0);
  assert.equal(report.ahead, 2);
  assert.equal(report.originalUnique, 1);
}));

test('rejects mismatched expected original-unique commit count', () => fixture(input => {
  git(input.cwd, 'switch', 'fork');
  git(input.cwd, 'merge', '--no-ff', '-m', 'integrate latest upstream', 'upstream');
  const result = run({ ...input, expected: 754 });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /expected 754.*found 1/i);
}));

test('rejects non-existent refs rather than treating them as zero commits', () => fixture(input => {
  const result = run({ ...input, candidate: 'does-not-exist' });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /cannot resolve.*does-not-exist/i);
}));
