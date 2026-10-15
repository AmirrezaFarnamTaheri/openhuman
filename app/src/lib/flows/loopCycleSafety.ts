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
  // Pinned TinyFlows validation requires exactly one trigger, whose run-level
  // recursion/visit budget applies to the entire run, even across distinct
  // SCCs. Trigger cardinality is validated authoritatively by tinyflows.
  const hasRunLevelBound = graph.nodes.some(node =>
    node.kind === 'trigger' &&
    ['recursion_limit', 'max_node_visits'].some(field => {
      const value = node.config?.[field];
      return typeof value === 'number' && Number.isSafeInteger(value) && value > 0;
    })
  );
  const outgoing = new Map<string, string[]>();
  const reverse = new Map<string, string[]>();
  // Keep original edge indices so diagnostics can be emitted in graph order
  // without rescanning the entire edge list for every cyclic component.
  const incomingIndices = new Map<string, number[]>();
  for (const id of ids) {
    outgoing.set(id, []);
    reverse.set(id, []);
    incomingIndices.set(id, []);
  }
  for (let index = 0; index < graph.edges.length; index++) {
    const edge = graph.edges[index];
    if (!ids.has(edge.from_node) || !ids.has(edge.to_node)) continue;
    outgoing.get(edge.from_node)!.push(edge.to_node);
    reverse.get(edge.to_node)!.push(edge.from_node);
    incomingIndices.get(edge.to_node)!.push(index);
  }

  // Match TinyFlows' per-node validation: an invalid loop config is a
  // problem even when that loop is not currently wired into a cycle.
  const issues: LoopSafetyIssue[] = [];
  for (const node of graph.nodes) {
    if (node.kind !== 'loop') continue;
    const configuredCap = node.config?.max_iterations;
    const maxIterations = configuredCap === undefined ? DEFAULT_LOOP_ITERATIONS : configuredCap;
    if (
      typeof maxIterations !== 'number' ||
      !Number.isInteger(maxIterations) ||
      maxIterations < 1 ||
      maxIterations > MAX_LOOP_ITERATIONS
    ) {
      issues.push({
        code: 'unbounded_loop',
        nodeIds: [node.id],
        message: 'Loop ' + node.id + ' must have max_iterations between 1 and ' + MAX_LOOP_ITERATIONS + '.',
      });
    }
  }

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
    // Each edge targets exactly one component. Visiting its incoming index
    // avoids O(cyclic SCCs * all edges), and sorting by the original index
    // preserves the user-authored ordering of independent diagnostics.
    const incident = members
      .flatMap(member => incomingIndices.get(member) ?? [])
      .sort((a, b) => a - b);
    for (const index of incident) {
      const edge = graph.edges[index];
      const member = edge.to_node;
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
