import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { fileURLToPath } from 'node:url';

const root = new URL('../../../', import.meta.url);
const read = path => readFileSync(new URL(path, root), 'utf8');

test('fork PRs run hosted lanes directly, with no membership API round trip', () => {
  const fast = read('.github/workflows/ci-fast-hosted.yml');
  assert.match(fast, /fork-lanes:\s*\n\s*name:.*\n\s*if: github\.repository != 'tinyhumansai\/openhuman'/);
  assert.match(fast, /fork-lanes:[\s\S]*?uses: \.\/\.github\/workflows\/ci-lanes\.yml[\s\S]*?profile: hosted/);
  assert.match(fast, /route:[\s\S]*?if: github\.repository == 'tinyhumansai\/openhuman'/);
  const org = read('.github/workflows/ci-fast.yml');
  assert.match(org, /route:[\s\S]*?if: github\.repository == 'tinyhumansai\/openhuman'/);
});

test('fork census retains on-demand control without frequent scheduled runner churn', () => {
  const census = read('.github/workflows/fork-ahead-census.yml');
  assert.match(census, /workflow_dispatch:/);
  assert.match(census, /cron: ['"]19 3 \* \* \*['"]/);
  assert.doesNotMatch(census, /19 \*\/2 \* \* \*/);
});
