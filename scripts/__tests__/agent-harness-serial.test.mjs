import assert from 'node:assert/strict';
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
    assert.match(
      e2e,
      /\[\s+"\$suite"\s+=\s+"agent_harness_e2e"\s+\][\s\S]{0,480}?--test-threads=1/,
      'Regular E2E must not race scripted completions and process-global state.'
    );
  });
});
