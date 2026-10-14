import { describe, expect, it } from 'vitest';
import { summarizeFleetHealth } from '../fleetHealthInsights';

describe('live model-fleet health summary', () => {
  it('does not treat missing observations as healthy keys or quota', () => {
    expect(summarizeFleetHealth({ providers: [] })).toEqual({
      providerCount: 0, modelCount: 0, keyCount: 0, healthy: 0, cooling: 0,
      evicted: 0, activeLeases: 0, nextRecoveryMs: null,
      successCount: 0, failureCount: 0, observedSuccessRate: null, status: 'unobserved',
    });
  });

  it('aggregates live key health with separate cooldown and lease counts', () => {
    const status = summarizeFleetHealth({ providers: [{slug: 'openai', models: [
      { model: 'gpt', keys: [
        { index: 0, status: 'healthy', successes: 2, failures: 0, active_leases: 3, cooldown_remaining_ms: 0 },
        { index: 1, status: 'cooldown', successes: 0, failures: 1, active_leases: 0, cooldown_remaining_ms: 1200 },
      ] },
      { model: 'gpt-vision', keys: [
        { index: 0, status: 'evicted', successes: 0, failures: 4, active_leases: 0, cooldown_remaining_ms: 0 },
      ] },
    ] }] });
    expect(status).toMatchObject({providerCount: 1, modelCount: 2, keyCount: 3,
      healthy: 1, cooling: 1, evicted: 1, activeLeases: 3,
      nextRecoveryMs: 1200, successCount: 2, failureCount: 5,
      observedSuccessRate: 29, status: 'degraded'});
  });

  it('never exposes raw diagnostic or key fields from the RPC payload', () => {
    const input = {providers: [{ slug: 'alpha', privateToken: 'do-not-render', models: [{model: 'x', keys: [
      {index: 0, status: 'healthy' as const, successes: 1, failures: 0,
        active_leases: 0, cooldown_remaining_ms: 0, privateToken: 'still-private'},
    ]}] }]};
    expect(JSON.stringify(summarizeFleetHealth(input))).not.toContain('do-not-render');
    expect(JSON.stringify(summarizeFleetHealth(input))).not.toContain('still-private');
  });
});

  it('does not infer an observation rate without attempts or from malformed counters', () => {
    const summary = summarizeFleetHealth({providers: [{slug:'google', models: [{model:'gemini',keys: [
      {index:0,status:'healthy',successes: Number.NaN, failures:-1,active_leases:0,cooldown_remaining_ms:0},
    ]}]}]});
    expect(summary).toMatchObject({
      successCount: 0, failureCount: 0, observedSuccessRate: null, status: 'ready',
    });
  });
