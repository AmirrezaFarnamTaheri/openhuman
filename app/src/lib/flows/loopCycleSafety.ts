/**
 * Local, deterministic soundness check for authored tinyflows loop cycles.
 * The core still owns authoritative workflow validation and execution.
 * Every cycle must consume the budget of some loop controller OR be
 * governed by TinyFlows' explicit trigger-level recursion/visit budget.
 * Merely finding a loop node somewhere in the SCC is insufficient.
 */
export interface LoopSafetyGraph {
  nodes: Array<{ id: string; kind: string; config?: Record<string, unknown> }>;
  edges: Array<{ from_node: string; from_port: string; to_node: string }>;
}

export interface LoopSafetyIssue {
  code:
    | 'cycle_without_bound'
    | 'unbounded_loop'
    | 'loop_side_entry'
    | 'loop_done_reenters'
    | 'loop_bypass_cycle';
  nodeIds: string[];
  message: string;
}

const MAX_LOOP_ITERATIONS = 100;
const DEFAULT_LOOP_ITERATIONS = 25;

export function validateLoopCycleSafety(graph: LoopSafetyGraph): LoopSafetyIssue[] {
  const ids = new Set(graph.nodes.map(node => node.id));
  const byId = new Map(graph.nodes.map(node => [node.id, node]));
  // TinyFlows supports run-level recursion_limit and max_node_visits as
  // alternatives to a loop controller. A budget belongs to its trigger's
  // execution, not to every disconnected component in the editor. We must
  // also reject cycles reachable from *any* unbounded trigger.
  const triggerHasBound = (node: LoopSafetyGraph['nodes'][number]): boolean =>
    ['recursion_limit', 'max_node_visits'].some(field => {
      const value = node.config?.[field];
      return typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
    });
  const outgoing = new Map<string, string[]>();
  const reverse = new Map<string, string[]>();
  const incoming = new Map<string, LoopSafetyGraph['edges']>();
  for (const id of ids) {
    outgoing.set(id, []);
    reverse.set(id, []);
    incoming.set(id, []);
  }
  for (const edge of graph.edges) {
    if (!ids.has(edge.from_node) || !ids.has(edge.to_node)) continue;
    outgoing.get(edge.from_node)!.push(edge.to_node);
    reverse.get(edge.to_node)!.push(edge.from_node);
    incoming.get(edge.to_node)!.push(edge);
  }

  // Multi-source traversal is O(nodes + edges) regardless of the number of
  // triggers. A single global boolean would incorrectly let an unrelated,
  // bounded trigger authorize another trigger's unbounded cycle.
  const reachableFrom = (starts: string[]): Set<string> => {
    const seen = new Set(starts);
    const queue = [...starts];
    for (let index = 0; index < queue.length; index++) {
      for (const next of outgoing.get(queue[index]) ?? []) {
        if (!seen.has(next)) {
          seen.add(next);
          queue.push(next);
        }
      }
    }
    return seen;
  };
  const boundedTriggers: string[] = [];
  const unboundedTriggers: string[] = [];
  for (const node of graph.nodes) {
    if (node.kind !== 'trigger') continue;
    (triggerHasBound(node) ? boundedTriggers : unboundedTriggers).push(node.id);
  }
  const boundedReachable = reachableFrom(boundedTriggers);
  const unboundedReachable = reachableFrom(unboundedTriggers);

  // Iterative Kosaraju avoids recursion exhaustion on large imported graphs.
  const visited = new Set<string>();
  const finishing: string[] = [];
  for (const root of ids) {
    if (visited.has(root)) continue;
    visited.add(root);
    const stack: Array<{ id: string; next: number }> = [{ id: root, next: 0 }];
    while (stack.length > 0) {
      const top = stack[stack.length - 1];
      const neighbors = outgoing.get(top.id)!;
      if (top.next < neighbors.length) {
        const next = neighbors[top.next++];
        if (!visited.has(next)) {
          visited.add(next);
          stack.push({ id: next, next: 0 });
        }
      } else {
        finishing.push(top.id);
        stack.pop();
      }
    }
  }

  const assigned = new Set<string>();
  const issues: LoopSafetyIssue[] = [];
  for (let i = finishing.length - 1; i >= 0; i--) {
    const root = finishing[i];
    if (assigned.has(root)) continue;
    assigned.add(root);
    const members: string[] = [];
    const queue = [root];
    while (queue.length > 0) {
      const current = queue.pop()!;
      members.push(current);
      for (const prev of reverse.get(current)!) {
        if (!assigned.has(prev)) {
          assigned.add(prev);
          queue.push(prev);
        }
      }
    }
    const memberSet = new Set(members);
    const cyclic = members.length > 1 || outgoing.get(root)!.includes(root);
    if (!cyclic) continue;
    members.sort();
    // Every entry path needs a bound. An unreachable SCC has no proven
    // trigger budget, and an unbounded trigger defeats a bounded peer.
    const hasRunLevelBound =
      members.some(id => boundedReachable.has(id)) &&
      !members.some(id => unboundedReachable.has(id));
    const loopNodes = members.filter(id => byId.get(id)?.kind === 'loop');
    const loopNodeIds = new Set(loopNodes);
    if (loopNodes.length === 0 && !hasRunLevelBound) {
      issues.push({
        code: 'cycle_without_bound',
        nodeIds: members,
        message: 'Cyclic component [' + members.join(', ') +
          '] needs a loop controller or a trigger recursion/visit bound.',
      });
      continue;
    }
    // An SCC may contain more than one controller: the native engine supports
    // that topology. Validate each configured controller independently.
    for (const head of loopNodes) {
      const configuredCap = byId.get(head)?.config?.max_iterations;
      const maxIterations = configuredCap === undefined ? DEFAULT_LOOP_ITERATIONS : configuredCap;
      if (
        typeof maxIterations !== 'number' ||
        !Number.isInteger(maxIterations) ||
        maxIterations < 1 ||
        maxIterations > MAX_LOOP_ITERATIONS
      ) {
        issues.push({
          code: 'unbounded_loop',
          nodeIds: [head],
          message: 'Loop ' + head + ' must have max_iterations between 1 and ' + MAX_LOOP_ITERATIONS + '.',
        });
      }
    }
    // Every edge has exactly one target component; scanning only incoming
    // edges to members avoids O(cyclic SCCs * all edges) on large workflows.
    for (const member of members) {
      for (const edge of incoming.get(member) ?? []) {
        if (
          loopNodes.length > 0 &&
          !memberSet.has(edge.from_node) &&
          !loopNodeIds.has(member)
        ) {
          issues.push({
            code: 'loop_side_entry',
            nodeIds: [edge.from_node, member],
            message: 'External flow must enter a cyclic component through a loop controller, not ' +
              member + '.',
          });
        }
        if (
          loopNodeIds.has(edge.from_node) &&
          edge.from_port === 'done'
        ) {
          issues.push({
            code: 'loop_done_reenters',
            nodeIds: [edge.from_node, member],
            message: 'Loop ' + edge.from_node + ' done port must exit its cyclic component.',
          });
        }
      }
    }
    if (hasRunLevelBound) continue;

    // All native loop nodes are removed, not just one. Every residual cycle
    // would bypass *every* controller and could repeat without consuming a
    // loop's iteration counter. A trigger-level bound (above) instead covers it.
    const residual = members.filter(id => !loopNodes.includes(id));
    const indegree = new Map(residual.map(id => [id, 0]));
    for (const node of residual) {
      for (const next of outgoing.get(node)!) {
        if (indegree.has(next)) indegree.set(next, indegree.get(next)! + 1);
      }
    }
    const ready = residual.filter(id => indegree.get(id) === 0);
    let removed = 0;
    while (ready.length > 0) {
      const node = ready.pop()!;
      removed++;
      for (const next of outgoing.get(node)!) {
        if (indegree.has(next)) {
          const value = indegree.get(next)! - 1;
          indegree.set(next, value);
          if (value === 0) ready.push(next);
        }
      }
    }
    if (removed !== residual.length) {
      const offenders = residual.filter(id => indegree.get(id)! > 0);
      issues.push({
        code: 'loop_bypass_cycle',
        nodeIds: offenders,
        message: 'Cyclic component contains a cycle bypassing every loop controller: [' +
          offenders.join(', ') + '].',
      });
    }
  }
  return issues;
}
