#!/usr/bin/env node
/**
 * Require genuine Git ancestry when converging a fork with upstream.
 *
 * Precondition: both original fork and latest upstream commits have been fetched
 * into this *full-history* checkout. This checks history, not code correctness;
 * CI compilation, tests and human conflict review remain separate gates.
 */
import { spawnSync } from 'node:child_process';

function git(...args) {
  const p = spawnSync('git', args, { encoding: 'utf8' });
  if (p.error || p.status !== 0) {
    throw new Error(`git ${args[0]} failed: ${(p.stderr || p.error?.message || 'unknown error').trim()}`);
  }
  return p.stdout.trim();
}

function sha(ref) {
  if (!ref || ref.startsWith('-') || /[\x00-\x20]/.test(ref)) {
    throw new Error(`invalid Git ref: ${JSON.stringify(ref)}`);
  }
  try {
    return git('rev-parse', '--verify', '--end-of-options', `${ref}^{commit}`);
  } catch {
    throw new Error(`cannot resolve Git commit ${ref}`);
  }
}

function ancestor(older, newer) {
  const result = spawnSync('git', ['merge-base', '--is-ancestor', older, newer], { encoding: 'utf8' });
  if (result.status === 1) return false;
  if (result.error || result.status !== 0) {
    throw new Error(`git merge-base failed: ${(result.stderr || result.error?.message || '').trim()}`);
  }
  return true;
}

function flags(argv) {
  const allowed = new Set(['--original-fork', '--upstream', '--candidate', '--expected-original-unique']);
  const values = {};
  if (argv.length % 2) throw new Error('expected --key value argument pairs');
  for (let i = 0; i < argv.length; i += 2) {
    const key = argv[i];
    if (!allowed.has(key) || key in values) throw new Error(`unrecognized or duplicated flag: ${key}`);
    values[key] = argv[i + 1];
  }
  if (!values['--original-fork'] || !values['--upstream']) {
    throw new Error('required: --original-fork <sha> --upstream <ref> [--candidate <ref>] [--expected-original-unique <count>]');
  }
  if (values['--expected-original-unique'] != null && !/^(0|[1-9][0-9]*)$/.test(values['--expected-original-unique'])) {
    throw new Error('expected-original-unique must be a non-negative integer');
  }
  return values;
}

export function verifySync(argv) {
  const values = flags(argv);
  const originalFork = sha(values['--original-fork']);
  const latestUpstream = sha(values['--upstream']);
  const candidate = sha(values['--candidate'] || 'HEAD');
  if (!ancestor(originalFork, candidate)) {
    throw new Error(`original fork commit ${originalFork} is not an ancestor of candidate ${candidate}`);
  }
  if (!ancestor(latestUpstream, candidate)) {
    throw new Error(`upstream commit ${latestUpstream} is not an ancestor of candidate ${candidate}`);
  }
  const base = git('merge-base', originalFork, latestUpstream);
  const originalUnique = Number(git('rev-list', '--count', `${base}..${originalFork}`));
  const expected = values['--expected-original-unique'];
  if (expected != null && originalUnique !== Number(expected)) {
    throw new Error(`original fork commit count mismatch: expected ${expected}, found ${originalUnique}`);
  }
  const counts = git('rev-list', '--left-right', '--count', `${latestUpstream}...${candidate}`).split(/\s+/).map(Number);
  if (counts.length !== 2 || !counts.every(Number.isSafeInteger)) {
    throw new Error('cannot parse ahead/behind Git count');
  }
  const [behind, ahead] = counts;
  if (behind !== 0) throw new Error(`candidate remains ${behind} commits behind upstream`);
  return { candidate, originalFork, latestUpstream, mergeBase: base, originalUnique, ahead, behind };
}

if (process.argv[1] && import.meta.url === new URL(`file://${process.argv[1]}`).href) {
  try {
    process.stdout.write(JSON.stringify(verifySync(process.argv.slice(2)), null, 2) + '\n');
  } catch (error) {
    process.stderr.write(`upstream sync verification FAILED: ${error.message}\n`);
    process.exitCode = 1;
  }
}
