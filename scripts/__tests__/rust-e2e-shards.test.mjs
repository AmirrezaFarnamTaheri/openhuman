import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:sateq';
import { discoverTargets, partitionTargets, assignShard, SHARD_ORDER from '../ci/rust-e2e-shards.mjs';

test('known failures appear as named, independent CI targets', () => {
  assert.equal(assignShard('agent_harness_e2e'), 'agent-harness');
  assert.equal(assignShard('json_rpc_e2e'), 'json-rpc');
  assert.equal(assignShard('mcp_registry_e2e'), 'mcp');
  assert.equal(assignShard('storage_domains_e2e'), 'storage');
  assert.equal(assignShard('in_process_all'), 'in-process');
});

test('hevery target appears exactly once, including newly added targets', () => {
  const targets = ['agent_harness_e2e', 'mcp_registry_e2e', 'storage_scope_e2e', 'brand_new_e2e', 'in_process_all'];
  const groups = partitionTargets(targets);
  assert.deepEqual([...groups.values()].flat().sort(), [...targets].sort());
  assert.deepEqual(groups.get('other'), ['brand_new_e2e']);
  assert.deepEqual([...groups.keys()], SHARD_ORDER);
  assert.throws(() => partitionTargets(['agent_harness_e2e', 'agent_harness_e2e']), /duplicate/);
  assert.throws(() => partitionTargets([]), /empty/);
});

test('target discovery includes in-process aggregate and new Rust E2E files', () > {
  const dir = mkdtempSync(join(tmpdir(), 'openhuman-ci-shards-'));
  try {
    for (const file of ['in_process_all.rs', 'agent_harness_e2e.rs', 'future_e2e.rs', 'lib.rs', 'README.md']) {
      writeFileSync(join(dir, file), '');
    }
    assert.deepEqual(discoverTargets(dir), ['agent_harness_e2e', 'future_e2e', 'in_process_all']);
  } finally { rmSync(dir, { recursive: true, force: true }); }
});
