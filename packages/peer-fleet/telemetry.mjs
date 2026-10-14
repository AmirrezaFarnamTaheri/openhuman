/** Credential-free attempt diagnostics adapted from cockpit-tools' request lifecycle.
 * Preserve exact upstream attempt identity, never prompt text, URLs or keys.
 */
const ID = /^[A-Za-z0-9][A-Za-z0-9._:/-]{0,199}$/;
function validTime(v) { return Number.isSafeInteger(v) && v>0; }
function quantile(sorted,p) { return sorted.length ? sorted[Math.max(0,Math.ceil(p*sorted.length)-1)] : null; }

/** One upstream attempt: conservative measured latency and *approximate* tokens/s. */
export function normalizeAttemptDiagnostic(row) {
  if (!row || !ID.test(row.provider??'') || !ID.test(row.model??'') || !ID.test(row.attemptId??'') ||
    !validTime(row.requestedAt) || !validTime(row.completedAt) || row.completedAt<row.requestedAt ||
    !Number.isSafeInteger(row.outputTokens) || row.outputTokens<0) return null;
  const latencyMs=row.completedAt-row.requestedAt;
  const failed=row.failed===true || row.upstreamComplete!==true;
  const eligible=row.stream===true && !failed && row.outputTokens>=100 &&
    validTime(row.firstFragmentAt) && row.firstFragmentAt>=row.requestedAt &&
    row.completedAt-row.firstFragmentAt>=100;
  const outputGenerationMs=eligible?row.completedAt-row.firstFragmentAt:null;
  const outputTokensPerSecond=outputGenerationMs===null?null:Math.round(row.outputTokens*1000/outputGenerationMs*100)/100;
  return {provider:row.provider,model:row.model,attemptId:row.attemptId,latencyMs,failed,
    outputTokens:row.outputTokens,outputGenerationMs,outputTokensPerSecond};
}

/** Roll up only sanitized data. Replays/duplicate events for one attempt count once. */
export function aggregateAttemptDiagnostics(rows) {
  const dedupe=new Set(), groups=new Map();
  for(const row of Array.isArray(rows)?rows:[]) {
    const event=normalizeAttemptDiagnostic(row);
    if(!event) continue;
    const key=`${event.provider}\u0000${event.attemptId}`;
    if(dedupe.has(key)) continue;
    dedupe.add(key);
    const groupKey=`${event.provider}\u0000${event.model}`;
    let g=groups.get(groupKey);
    if(!g){g={provider:event.provider,model:event.model,attempts:0,successful:0,failed:0,latencies:[],rates:[]};groups.set(groupKey,g)}
    g.attempts++;
    if(event.failed)g.failed++;
    else {g.successful++;g.latencies.push(event.latencyMs)}
    if(event.outputTokensPerSecond!==null)g.rates.push(event.outputTokensPerSecond);
  }
  return [...groups.values()].map(g=>{
    g.latencies.sort((a,b)=>a-b);g.rates.sort((a,b)=>a-b);
    return {provider:g.provider,model:g.model,attempts:g.attempts,successful:g.successful,failed:g.failed,
      p50LatencyMs:quantile(g.latencies,.5),p90LatencyMs:quantile(g.latencies,.9),
      p50ApproxOutputTokensPerSecond:quantile(g.rates,.5)};
  }).sort((a,b)=>a.provider.localeCompare(b.provider)||a.model.localeCompare(b.model));
}
