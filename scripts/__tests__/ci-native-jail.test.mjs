import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { parse } from 'yaml';

const read = path => readFileSync(new URL(`../../${path}`, import.meta.url), 'utf8');

test('Rust CI prepares a real nested jail without privileged containers', () => {
  for (const [file, job] of [
    ['rust-e2e-shards.yml', 'shard'],
    ['ci-lanes.yml', 'lanes'],
    ['test-reusable.yml', 'rust-core-tests'],
  ]) {
    const lane = parse(read(`.github/workflows/${file}`)).jobs[job];
    assert.match(lane.container.options, /--init/);
    assert.match(lane.container.options, /--security-opt seccomp=unconfined/);
    assert.match(lane.container.options, /--security-opt apparmor=unconfined/);
    assert.doesNotMatch(lane.container.options, /--privileged|--cap-add/);
    assert.ok(lane.steps.some(step => step.run === 'bash scripts/ci/prepare-native-jail.sh'));
  }
});

test('the native jail preflight fails closed and the image includes bubblewrap', () => {
  const script = read('scripts/ci/prepare-native-jail.sh');
  assert.match(script, /set -euo pipefail/);
  assert.match(script, /apt-get install -y --no-install-recommends bubblewrap/);
  assert.match(script, /bwrap --unshare-user --unshare-pid --unshare-net/);
  assert.doesNotMatch(script, /\|\| true|SANDBOX.*(false|off)/);
  assert.match(read('.github/Dockerfile'), /\n    bubblewrap /);
});
