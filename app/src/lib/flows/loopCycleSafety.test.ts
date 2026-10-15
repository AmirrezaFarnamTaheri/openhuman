import { describe, expect, it } from 'vitest';
import { validateLoopCycleSafety } from './loopCycleSafety';

const node = (id: string, kind = 'agent', config: Record<string, unknown> = {}) =>
  ({ id, kind, config });
const edge = (from_node: string, to_node: string, from_port = 'main') =>
  ({ from_node, to_node, from_port });

describe('validateLoopCycleSafety', () => {
  it('accepts ordinary acyclic flows and bounded header-controlled loops', () => {
    expect(validateLoopCycleSafety({
      nodes: [node('trigger'), node('task')],
      edges: [edge('trigger', 'task')],
    })).toEqual([]);
    expect(validateLoopCycleSafety({
      nodes: [node('loop', 'loop'), node('task'), node('exit')],
      edges: [edge('loop', 'task', 'body'), edge('task', 'loop'), edge('loop', 'exit', 'done')],
    })).toEqual([]);
  });

  it('rejects cycles that have neither a loop controller nor an explicit run bound', () => {
    expect(validateLoopCycleSafety({
      nodes: [node('a'), node('b')],
      edges: [edge('a', 'b'), edge('b', 'a')],
    }).map(issue => issue.code)).toEqual(['cycle_without_bound']);
  });

  it('accepts a run-level visit budget instead of a controller, as TinyFlows does', () => {
    const nodes = [
      node('t', 'trigger', { max_node_visits: 12 }),
      node('a'),
      node('b'),
    ];
    expect(validateLoopCycleSafety({
      nodes,
      edges: [edge('t', 'a'), edge('a', 'b'), edge('b', 'a')],
    })).toEqual([]);
    expect(validateLoopCycleSafety({
      nodes: [node('t', 'trigger', { recursion_limit: 5 }), node('a')],
      edges: [edge('t', 'a'), edge('a', 'a')],
    })).toEqual([]);
  });


  it('does not let a disconnected bounded trigger authorize an unrelated cycle', () => {
    const issues = validateLoopCycleSafety({
      nodes: [
        node('safe-trigger', 'trigger', { max_node_visits: 10 }),
        node('safe-task'),
        node('unsafe-trigger', 'trigger'),
        node('a'), node('b'),
      ],
      edges: [
        edge('safe-trigger', 'safe-task'),
        edge('unsafe-trigger', 'a'), edge('a', 'b'), edge('b', 'a'),
      ],
    });
    expect(issues.map(issue => issue.code)).toEqual(['cycle_without_bound']);
    expect(issues[0].nodeIds).toEqual(['a', 'b']);
  });

  it('refuses a cycle reachable through any unbounded trigger even if another is bounded', () => {
    const issues = validateLoopCycleSafety({
      nodes: [
        node('bounded', 'trigger', { recursion_limit: 5 }),
        node('unbounded', 'trigger'), node('a'), node('b'),
      ],
      edges: [
        edge('bounded', 'a'), edge('unbounded', 'b'),
        edge('a', 'b'), edge('b', 'a'),
      ],
    });
    expect(issues.map(issue => issue.code)).toEqual(['cycle_without_bound']);
  });

  it('allows run-level bounds only for cycles reachable exclusively from bounded triggers', () => {
    const issues = validateLoopCycleSafety({
      nodes: [
        node('t1', 'trigger', { max_node_visits: 10 }),
        node('t2', 'trigger', { recursion_limit: 3 }),
        node('unrelated', 'trigger'), node('a'), node('b'),
      ],
      edges: [
        edge('t1', 'a'), edge('t2', 'b'),
        edge('a', 'b'), edge('b', 'a'),
      ],
    });
    expect(issues).toEqual([]);
  });

  it('accepts multiple loop controllers when every cycle crosses a controller', () => {
    expect(validateLoopCycleSafety({
      nodes: [node('a', 'loop'), node('b', 'loop')],
      edges: [edge('a', 'b', 'body'), edge('b', 'a', 'body')],
    })).toEqual([]);
  });

  it('keeps bypass-cycle detection even when multiple loop controllers are present', () => {
    expect(validateLoopCycleSafety({
      nodes: [node('headA', 'loop'), node('headB', 'loop'), node('a'), node('b')],
      edges: [
        edge('headA', 'headB', 'body'), edge('headB', 'a', 'body'),
        edge('a', 'b'), edge('b', 'a'), edge('b', 'headA'),
      ],
    }).map(issue => issue.code)).toEqual(['loop_bypass_cycle']);
  });

  it('rejects a hidden cycle that bypasses the bounded loop controller', () => {
    const issues = validateLoopCycleSafety({
      nodes: [node('loop', 'loop'), node('a'), node('b')],
      edges: [
        edge('loop', 'a', 'body'), edge('a', 'b'), edge('b', 'a'), edge('b', 'loop'),
      ],
    });
    expect(issues.map(issue => issue.code)).toEqual(['loop_bypass_cycle']);
    expect(issues[0].nodeIds).toContain('a');
  });

  it('rejects side entries and a done-port edge remaining inside a cycle', () => {
    expect(validateLoopCycleSafety({
      nodes: [node('loop', 'loop'), node('a'), node('outside')],
      edges: [edge('loop', 'a', 'body'), edge('a', 'loop'), edge('outside', 'a')],
    }).map(issue => issue.code)).toEqual(['loop_side_entry']);
    expect(validateLoopCycleSafety({
      nodes: [node('loop', 'loop'), node('a')],
      edges: [edge('loop', 'a', 'done'), edge('a', 'loop')],
    }).map(issue => issue.code)).toEqual(['loop_done_reenters']);
  });

  it('bounds iterations and preserves the engine default when omitted', () => {
    for (const cap of [0, -1, 101, 1.5, Number.NaN, 'forever']) {
      expect(validateLoopCycleSafety({
        nodes: [node('loop', 'loop', { max_iterations: cap }), node('task')],
        edges: [edge('loop', 'task', 'body'), edge('task', 'loop')],
      }).map(issue => issue.code)).toEqual(['unbounded_loop']);
    }
    expect(validateLoopCycleSafety({
      nodes: [node('loop', 'loop', { max_iterations: 1 }), node('task')],
      edges: [edge('loop', 'task', 'body'), edge('task', 'loop')],
    })).toEqual([]);
  });

  it('rejects a self-cycle without a loop', () => {
    expect(validateLoopCycleSafety({
      nodes: [node('a')], edges: [edge('a', 'a')],
    }).map(issue => issue.code)).toEqual(['cycle_without_bound']);
  });

  it('validates a 14k-node imported DAG without recursion overflow', () => {
    const nodes = Array.from({ length: 14000 }, (_, index) => node('n' + index));
    const edges = Array.from({ length: 13999 }, (_, index) =>
      edge('n' + index, 'n' + (index + 1)));
    expect(validateLoopCycleSafety({ nodes, edges })).toEqual([]);
  });
});
