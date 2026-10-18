import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { runInNewContext } from 'node:vm';
import { parse as parseYaml } from 'yaml';

const root = new URL('../../../', import.meta.url);
const read = path => readFileSync(new URL(path, root), 'utf8');

const hosted = () => parseYaml(read('.github/workflows/ci-fast-hosted.yml'));

function runsHosted(repository, result, outsider = '', cancelled = false) {
  const condition = hosted().jobs.lanes.if;
  // This workflow uses only string/boolean comparisons and the status function.
  // Evaluate its actual predicate; do not duplicate the routing logic here.
  return runInNewContext(`(${condition})`, {
    github: { repository },
    needs: { route: { result, outputs: { outsider } } },
    cancelled: () => cancelled,
  }, { timeout: 1000 });
}

test('fork PRs skip membership lookup and run hosted lanes after a skipped route', () => {
  const fast = hosted();
  assert.equal(fast.jobs.route.if, "github.repository == 'tinyhumansai/openhuman'");
  assert.equal(fast.jobs.lanes.needs, 'route');
  // A status function overrides GitHub's implicit success() after skipped needs.
  assert.match(fast.jobs.lanes.if, /!cancelled\(\)/);
  assert.equal(runsHosted('contributor/openhuman', 'skipped'), true);
  const org = read('.github/workflows/ci-fast.yml');
  assert.match(org, /route:[\s\S]*?if: github\.repository == 'tinyhumansai\/openhuman'/);
});

test('upstream hosted lanes require a successful outsider routing result', () => {
  assert.equal(runsHosted('tinyhumansai/openhuman', 'success', 'true'), true);
  for (const outsider of ['false', '']) {
    assert.equal(runsHosted('tinyhumansai/openhuman', 'success', outsider), false);
  }
  for (const result of ['failure', 'cancelled', 'skipped', 'timed_out']) {
    assert.equal(runsHosted('tinyhumansai/openhuman', result, 'true'), false, result);
  }
});

test('cancelled workflows cannot start either hosted route', () => {
  assert.equal(runsHosted('contributor/openhuman', 'skipped', '', true), false);
  assert.equal(runsHosted('tinyhumansai/openhuman', 'success', 'true', true), false);
});

test('hosted callers preserve the decisive name understood by the default-branch gate', () => {
  const callers = Object.values(hosted().jobs).filter(
    job => job.uses === './.github/workflows/ci-lanes.yml',
  );
  assert.equal(callers.length, 1, 'one caller prevents competing skipped gate names');
  const shared = parseYaml(read('.github/workflows/ci-lanes.yml'));
  assert.equal(`${callers[0].name} / ${shared.jobs.gate.name}`, 'Lanes / CI Fast (hosted) Gate');
  assert.equal(callers[0].with.profile, 'hosted');
});

test('fork census retains on-demand control without frequent scheduled runner churn', () => {
  const census = read('.github/workflows/fork-ahead-census.yml');
  assert.match(census, /workflow_dispatch:/);
  assert.match(census, /cron: ['"]19 3 \* \* \*['"]/);
  assert.doesNotMatch(census, /19 \*\/2 \* \* \*/);
});
