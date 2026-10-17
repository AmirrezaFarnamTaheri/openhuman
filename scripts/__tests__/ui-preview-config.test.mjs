import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';

const source = () => readFileSync(new URL('../../.github/workflows/ui-preview.yml', import.meta.url), 'utf8');

test('unconfigured optional UI preview skips before expensive app or browser builds', () => {
  const workflow = source();
  assert.match(workflow, /github\.event\.pull_request\.head\.repo\.full_name == github\.repository\s*&&\s*vars\.TINYSWEEPER_SERVER_URL != ''/);
  assert.match(workflow, /name: Build the standalone core once/);
  assert.match(workflow, /name: Drive the preview/);
});

test('configured UI preview still has the same pull request trigger and file scope', () => {
  const workflow = source();
  assert.match(workflow, /pull_request:\s*\n\s*types: \[opened, synchronize, reopened, ready_for_review\]/);
  assert.match(workflow, /- "app\/src\/\*\*"/);
  assert.match(workflow, /TS_SERVER: \$\{\{ vars\.TINYSWEEPER_SERVER_URL \}\}/);
});
