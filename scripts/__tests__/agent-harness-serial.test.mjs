import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { readFileSync } from 'node:fs';
import { describe, it } from 'node:test';

const coverage = readFileSync(new URL('../ci/rust-coverage.sh', import.meta.url), 'utf8');
const e2e = readFileSync(new URL('../test-rust-e2e.sh', import.meta.url), 'utf8');

describe('agent_harness_e2e CI isolation', () => {
  it('runs the harness coverage target with one libtest thread rather than alongside unrelated fixtures', () => {
    assert.match(
      coverage,
      /elif\s+\[\s+"\$\{target\}"\s+=\s+"agent_harness_e2e"\s+\](?:(?!\n\s*elif\b)[\s\S])*?--test-threads=1/,
      'Coverage must explicitly serialize process-global harness fixtures.'
    );
  });

  it('uses the same serial policy in ordinary Rust E2E runs', () => {
    const serialBranch = e2e.match(
      /if \[ "\$suite" = "mcp_registry_e2e" \] \|\| \[ "\$suite" = "agent_harness_e2e" \]; then([\s\S]*?)\n  fi/
    );
    assert.ok(serialBranch, 'Agent harness must run in the explicitly serialized branch.');
    assert.match(serialBranch[1], /filter_serial_libtest_args/);
    assert.match(serialBranch[1], /--test-threads=1/);
  });
});

const serialArgsHelper = new URL('../ci/serial-libtest-args.sh', import.meta.url);

function filterSerialArgs(...args) {
  const script = [
    'set -euo pipefail',
    'source "$1"',
    'shift',
    'EXTRA_ARGS=("$@")',
    'filter_serial_libtest_args',
    'printf "%s\\0" "${SERIAL_LIBTEST_ARGS[@]}"',
  ].join('; ');
  return spawnSync('bash', ['-c', script, 'bash', serialArgsHelper.pathname, ...args], {
    encoding: 'utf8',
  });
}

describe('serial libtest passthrough', () => {
  it('removes both --test-threads syntaxes and preserves unrelated arguments', () => {
    const result = filterSerialArgs('--nocapture', '--test-threads=8', '--exact', '--test-threads', '4', 'test_filter');
    assert.equal(result.status, 0, result.stderr);
    assert.deepEqual(result.stdout.split('\0').filter(Boolean), ['--nocapture', '--exact', 'test_filter']);
  });

  it('rejects a dangling --test-threads instead of swallowing a bad command line', () => {
    const result = filterSerialArgs('--exact', '--test-threads');
    assert.equal(result.status, 2);
    assert.match(result.stderr, /requires a value/i);
  });

  it('wires the filter into the real serial launcher', () => {
    assert.ok(e2e.includes('source "$SCRIPT_DIR/ci/serial-libtest-args.sh"'));
    assert.ok(e2e.includes('filter_serial_libtest_args'));
    assert.ok(e2e.includes('--test-threads=1 ${SERIAL_LIBTEST_ARGS[@]+'));
  });
});
