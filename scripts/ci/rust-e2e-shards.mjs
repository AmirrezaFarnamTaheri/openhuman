#!/usr/bin/env node
// Partition the complete Rust E2E target inventory into independent CI jobs.
// New `tests/*_e2e.rs` targets automatically land in `other`; no suite can
// silently disappear when its owner forgets to update a static matrix.
import { readdirSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const repo = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
export const SHARD_ORDER = [
  'agent-harness',
  'json-rpc',
  'mcp',
  'storage',
  'in-process',
  'providers',
  'other',
];

export function assignShard(target) {
  if (target === 'agent_harness_e2e') return 'agent-harness';
  if (target === 'json_rpc_e2e') return 'json-rpc';
  if (target === 'in_process_all') return 'in-process';
  if (target.startsWith('storage_') || target === 'cli_storage_url_e2e') return 'storage';
  if (/^(mcp_|tinymcp_|tinydocs_|tinysearch_)/.test(target)) return 'mcp';
  if (/^(inference_|cli_model_|live_routing_|saas_|media_generation_)/.test(target)) return 'providers';
  return 'other';
}

export function discoverTargets(testsDir = resolve(repo, 'tests')) {
  return readdirSync(testsDir)
    .filter((file) => /_e2e\.rs$/.test(file) || file === 'in_process_all.rs')
    .map((file) => file.replace(/\.rs$/, ''))
    .sort();
}

export function partitionTargets(targets) {
  const output = new Map(SHARD_ORDER.map((s) => [s, []]));
  if (targets.length === 0 || new Set(targets).size !== targets.length) {
    throw new Error('Rust E2E inventory is empty or contains duplicate targets');
  }
  for (const target of targets) {
    const group = assignShard(target);
    if (!output.has(group)) throw new Error(`Unknown Rust E2E shard: ${group}`);
    output.get(group).push(target);
  }
  const assigned = [...output.values()].flat();
  if (assigned.length !== targets.length || new Set(assigned).size !== targets.length) {
    throw new Error('Rust E2E shard inventory lost or duplicated a target');
  }
  return output;
}

function main() {
  const [option, argument] = process.argv.slice(2);
  const partition = partitionTargets(discoverTargets());
  if (option === '--matrix') {
    process.stdout.write(JSON.stringify({ include: [...partition]
      .filter(([, targets]) => targets.length > 0)
      .map(([group, targets]) => ({ group, count: targets.length })) }) + '\n');
  } else if (option === '--group' && partition.has(argument)) {
    const targets = partition.get(argument);
    if (!targets.length) throw new Error(`Rust E2E shard ${argument} is empty`);
    process.stdout.write(targets.join('\n') + '\n');
  } else if (option === '--verify') {
    for (const [group, targets] of partition) {
      if (targets.length) process.stderr.write(`[rust-e2e-shards] ${group}: ${targets.join(', ')}\n`);
    }
    process.stderr.write(`[rust-e2e-shards] accounted for ${discoverTargets().length} targets\n`);
  } else {
    throw new Error('Usage: rust-e2e-shards.mjs --matrix | --group <name> | --verify');
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(); } catch (error) { console.error(`[rust-e2e-shards] ${error.message}`); process.exitCode = 1; }
}
