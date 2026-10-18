import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { parse as parseYaml } from 'yaml';

const source = () => parseYaml(readFileSync(new URL('../../.github/workflows/ui-preview.yml', import.meta.url), 'utf8'));

test('unconfigured optional UI preview skips before expensive app or browser builds', () => {
  const workflow = source();
  const job = workflow.jobs['ui-preview'];
  assert.match(job.if, /github\.event\.pull_request\.draft == false\s*&&/);
  assert.match(job.if, /github\.event\.pull_request\.head\.repo\.full_name == github\.repository\s*&&\s*vars\.TINYSWEEPER_SERVER_URL != ''/);
  assert.ok(job.steps.some(step => step.name === 'Build the standalone core once'));
  assert.ok(job.steps.some(step => step.name === 'Drive the preview'));
});

test('configured UI preview still has the same pull request trigger and file scope', () => {
  const workflow = source();
  assert.deepEqual(workflow.on.pull_request.types, ['opened', 'synchronize', 'reopened', 'ready_for_review']);
  assert.ok(workflow.on.pull_request.paths.includes('app/src/**'));
  const drive = workflow.jobs['ui-preview'].steps.find(step => step.name === 'Drive the preview');
  assert.equal(drive.env.TS_SERVER, '${{ vars.TINYSWEEPER_SERVER_URL }}');
});
