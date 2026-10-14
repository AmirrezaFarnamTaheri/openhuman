/**
 * Local, deterministic soundness check for authored tinyflows loop cycles.
 * The core still owns authoritative workflow validation and execution.
 * A loop controller must dominate every cycle in its strongly connected
 * component; merely finding one loop node in an SCC is insufficient.
 */
export interface LoopSafetyGraph {
  nodes: Array<{ id: string; kind: string; config?: Record<string, unknown> }>;
  edges: Array<{ from_node: string; from_port: string; to_node: string }>;
}

export interface LoopSafetyIssue {
  code:
    | 'cycle_without_single_loop'
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
  const outgoing = new Map<string, string[]>();
  const reverse = new Map<string, string[]>();
  for (const id of ids) {
    outgoing.set(id, []);
    reverse.set(id, []);
  }
  for (const edge of graph.edges) {
    if (!ids.has(edge.from_node) || !ids.has(edge.to_node)) continue;
    outgoing.get(edge.from_node)!.push(edge.to_node);
    reverse.get(edge.to_node)!.push(edge.from_node);
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
    const loopNodes = members.filter(id => byId.get(id)?.kind === 'loop');
    if (loopNodes.length !== 1) {
      issues.push({
        code: 'cycle_without_single_loop',
        nodeIds: members,
        message: 'Cyclic component [' + members.join(', ') +
          '] must contain exactly one loop controller (found ' + loopNodes.length + ').',
      });
      continue;
    }
    const head = loopNodes[0];
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
    for (const edge of graph.edges) {
      if (memberSet.has(edge.to_node) && !memberSet.has(edge.from_node) && edge.to_node !== head) {
        issues.push({
          code: 'loop_side_entry',
          nodeIds: [edge.from_node, edge.to_node],
          message: 'External flow must enter the cyclic component through loop ' + head +
            ', not ' + edge.to_node + '.',
        });
      }
      if (edge.from_node === head && memberSet.has(edge.to_node) && edge.from_port === 'done') {
        issues.push({
          code: 'loop_done_reenters',
          nodeIds: [head, edge.to_node],
          message: 'Loop ' + head + ' done port must exit its cyclic component.',
        });
      }
    }

    // Removing the loop head removes every allowed back-edge. If the residual
    // graph is cyclic, a path can repeat indefinitely without consuming the
    // loop's iteration budget (a flaw in SCC-count-only validators).
    const residual = members.filter(id => id !== head);
    const indegree = new Map(residual.map(id => [id, 0]));
    for (const node of residual) {
      for (const next of outgoing.get(node)!) {
        if (next !== head && memberSet.has(next)) indegree.set(next, indegree.get(next)! + 1);
      }
    }
    const ready = residual.filter(id => indegree.get(id) === 0);
    let removed = 0;
    while (ready.length > 0) {
      const node = ready.pop()!;
      removed++;
      for (const next of outgoing.get(node)!) {
        if (next !== head && memberSet.has(next)) {
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
        message: 'Cyclic component contains a cycle bypassing loop ' + head +
          ': [' + offenders.join(', ') + '].',
      });
    }
  }
  return issues;
}
