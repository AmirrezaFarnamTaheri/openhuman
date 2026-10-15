import { describe, expect, it } from 'vitest';

import { assessModelFeasibility, type FeasibilityModel, type ModelRequirements } from './modelFeasibility';

const model = (id: string, fields: Partial<FeasibilityModel> = {}): FeasibilityModel => ({
  provider: 'demo', id, context_window: 64_000, vision: false,
  cost_per_1m_input: 1, cost_per_1m_output: 4, ...fields,
});
const req = (fields: Partial<ModelRequirements> = {}): ModelRequirements => ({
  inputTokens: 1000, outputTokens: 1000, requiresVision: false,
  maxEstimatedCostUSD: null, ...fields,
});

describe('saved-model suitability preview (advisory, no live routing)', () => {
  it('uses hard context and vision constraints before any price ranking', () => {
    const rows = assessModelFeasibility([
      model('a-short', { context_window: 1500, vision: true }),
      model('b-blind', { vision: false }),
      model('c-fit', { vision: true }),
    ], req({ requiresVision: true })).rows;
    expect(rows.map(row => row.model.id)).toEqual(['c-fit', 'a-short', 'b-blind']);
    expect(rows.map(row => row.status)).toEqual(['metadata_fit', 'excluded', 'excluded']);
  });

  it('does not mistake missing, invalid or zero prices for verified free use', () => {
    const rows = assessModelFeasibility([
      model('missing', { cost_per_1m_output: undefined }),
      model('zero', { cost_per_1m_input: 0 }),
      model('invalid', { cost_per_1m_input: NaN }),
      model('priced'),
    ], req()).rows;
    expect(rows.find(row => row.model.id === 'priced')?.status).toBe('metadata_fit');
    for (const id of ['missing', 'zero', 'invalid']) {
      const row = rows.find(candidate => candidate.model.id === id);
      expect(row?.status).toBe('unknown');
      expect(row?.estimatedCostUSD).toBeNull();
      expect(row?.reasons).toContain('unknown_price');
    }
  });

  it('excludes models over a specified budget while sorting remaining candidates by estimated cost', () => {
    const report = assessModelFeasibility([
      model('expensive', { cost_per_1m_input: 100, cost_per_1m_output: 100 }),
      model('cheap', { cost_per_1m_input: 0.5, cost_per_1m_output: 2 }),
      model('middle', { cost_per_1m_input: 1, cost_per_1m_output: 3 }),
    ], req({ maxEstimatedCostUSD: 0.01 }));
    expect(report.rows.map(row => row.model.id)).toEqual(['cheap', 'middle', 'expensive']);
    expect(report.counts).toEqual({ metadata_fit: 2, unknown: 0, excluded: 1 });
    expect(report.availabilityVerified).toBe(false);
    expect(report.livePricesVerified).toBe(false);
    expect(report.selectionIsAdvisory).toBe(true);
  });

  it('keeps missing context or vision capability unknown, without interpreting unknown as eligible', () => {
    const report = assessModelFeasibility([
      model('missing-context', { context_window: undefined, vision: true }),
      model('missing-vision', { vision: null }),
      model('fit', { vision: true }),
    ], req({ requiresVision: true }));
    expect(report.rows.find(row => row.model.id === 'missing-context')?.status).toBe('unknown');
    expect(report.rows.find(row => row.model.id === 'missing-vision')?.status).toBe('unknown');
    expect(report.rows[0].model.id).toBe('fit');
  });

  it('rejects invalid requirements and remains stable under reordered equal-price candidates', () => {
    for (const fields of [
      { inputTokens: -1 }, { outputTokens: Number.POSITIVE_INFINITY },
      { maxEstimatedCostUSD: -1 }, { inputTokens: 0.1 },
    ]) {
      expect(() => assessModelFeasibility([model('x')], req(fields))).toThrow(/Invalid/);
    }
    const first = [model('z'), model('b'), model('a')];
    const a = assessModelFeasibility(first, req());
    const b = assessModelFeasibility([...first].reverse(), req());
    expect(a.rows.map(row => row.model.id)).toEqual(['a', 'b', 'z']);
    expect(a.rows.map(row => row.model.id)).toEqual(b.rows.map(row => row.model.id));
  });
});
