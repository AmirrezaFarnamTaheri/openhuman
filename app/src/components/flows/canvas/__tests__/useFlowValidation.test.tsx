import { act, renderHook } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import type { WorkflowGraph } from '../../../../lib/flows/types';
import { useFlowValidation } from '../useFlowValidation';

const validateFlow = vi.hoisted(() => vi.fn());
vi.mock('../../../../services/api/flowsApi', () => ({ validateFlow }));

function graph(name: string): WorkflowGraph {
  return {
    schema_version: 1,
    name,
    nodes: [{ id: 'start', kind: 'trigger', name: 'Start', config: {}, ports: [] }],
    edges: [],
  };
}

const valid = { valid: true, errors: [], warnings: [] };
const invalid = { valid: false, errors: ['bad config'], warnings: [] };

describe('useFlowValidation draft revision isolation', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    validateFlow.mockReset();
  });
  afterEach(() => {
    vi.useRealTimers();
  });

  it('does not let a previous draft RPC validate a newly edited draft before its debounce', async () => {
    let resolveOld!: (value: typeof invalid) => void;
    validateFlow.mockImplementationOnce(() => new Promise(resolve => { resolveOld = resolve; }));
    validateFlow.mockResolvedValueOnce(valid);

    const { result, rerender } = renderHook(
      ({ draft, key }) => useFlowValidation(draft, key),
      { initialProps: { draft: graph('old'), key: 'old' } }
    );

    let oldRequest!: Promise<typeof invalid | null>;
    act(() => { oldRequest = result.current.validateNow(); });
    expect(result.current.validating).toBe(true);

    rerender({ draft: graph('new'), key: 'new' });
    expect(result.current.validation).toBeNull();
    expect(result.current.validating).toBe(false);

    await act(async () => {
      resolveOld(invalid);
      expect(await oldRequest).toBeNull();
    });
    expect(result.current.validation).toBeNull();

    await act(async () => { await result.current.validateNow(); });
    expect(result.current.validation).toEqual(valid);
    expect(result.current.validating).toBe(false);
  });

  it('clears already displayed validation synchronously when the draft key changes', async () => {
    validateFlow.mockResolvedValueOnce(invalid);
    const { result, rerender } = renderHook(
      ({ draft, key }) => useFlowValidation(draft, key),
      { initialProps: { draft: graph('A'), key: 'A' } }
    );
    await act(async () => { await result.current.validateNow(); });
    expect(result.current.validation).toEqual(invalid);
    rerender({ draft: graph('B'), key: 'B' });
    expect(result.current.validation).toBeNull();
    expect(result.current.validating).toBe(false);
  });

  it('retains the newest result on the same draft when two manual validations resolve backwards', async () => {
    let resolveFirst!: (value: typeof invalid) => void;
    let resolveSecond!: (value: typeof valid) => void;
    validateFlow.mockImplementationOnce(() => new Promise(resolve => { resolveFirst = resolve; }));
    validateFlow.mockImplementationOnce(() => new Promise(resolve => { resolveSecond = resolve; }));

    const { result } = renderHook(() => useFlowValidation(graph('A'), 'A'));
    let first!: Promise<typeof invalid | null>;
    let second!: Promise<typeof valid | null>;
    act(() => {
      first = result.current.validateNow();
      second = result.current.validateNow();
    });
    await act(async () => {
      resolveSecond(valid);
      expect(await second).toEqual(valid);
    });
    await act(async () => {
      resolveFirst(invalid);
      expect(await first).toBeNull();
    });
    expect(result.current.validation).toEqual(valid);
    expect(result.current.validating).toBe(false);
  });
});
