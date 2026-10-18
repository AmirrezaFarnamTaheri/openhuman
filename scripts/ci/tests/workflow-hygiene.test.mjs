import assert from 'node:assert/strict';
import { existsSync, readFileSync } from 'node:fs';
import { test } from 'node:test';

const root = new URL('../../../', import.meta.url);
const path = file => new URL(file, root);

test('no obsolete single-use or disabled package-build workflows remain registered', () => {
  for (const file of [
    '.github/workflows/pr6-kernel-diagnostic.yml',
    '.github/workflows/release-packages.yml',
  ]) {
    assert.equal(existsSync(path(file)), false, `${file} should be historical Git data, not active workflow chrome`);
  }
});

test('soft PR checklist and coverage checks share one checkout without losing either check', () => {
  const quality = readFileSync(path('.github/workflows/pr-quality.yml'), 'utf8');
  assert.match(quality, /metadata-checks:/);
  assert.doesNotMatch(quality, /^  checklist-guard:/m);
  assert.doesNotMatch(quality, /^  coverage-matrix:/m);
  assert.match(quality, /name: Verify Submission Checklist/);
  assert.match(quality, /name: Verify Coverage Matrix/);
});

test('release, PR, security and platform gates are preserved after consolidation', () => {
  for (const file of [
    '.github/workflows/ci-full.yml',
    '.github/workflows/ci-lanes.yml',
    '.github/workflows/ci-gate.yml',
    '.github/workflows/test-reusable.yml',
    '.github/workflows/build-desktop.yml',
    '.github/workflows/ios-compile.yml',
    '.github/workflows/android-compile.yml',
    '.github/workflows/pr-quality.yml',
    '.github/workflows/module-bundle.yml',
  ]) {
    assert.equal(existsSync(path(file)), true, `retained gate ${file}`);
  }
  assert.match(readFileSync(path('.github/workflows/ci-lanes.yml'), 'utf8'), /workflow_call:/);
});
