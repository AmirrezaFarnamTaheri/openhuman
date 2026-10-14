/** Framework-independent view models: React/Tauri/CLI adapters can render one contract. */
import {normalizeQuota,quotaRemaining,quotaColumns} from './quota.mjs';
export function buildDashboard(accounts,{now=Date.now()/1000}={}){
  if(!Array.isArray(accounts))throw new TypeError('accounts required');
  const rows=accounts.map(a=>{
    const quota=normalizeQuota(a.quota??{fetchedAt:null,windows:[]},now);
    return {id:a.id,provider:a.provider,health:a.health??'unknown',quota,remainingPercent:quotaRemaining(quota)};
  });
  return {summary:{total:rows.length,fresh:rows.filter(r=>r.quota.freshness==='fresh').length,stale:rows.filter(r=>r.quota.freshness==='stale').length,critical:rows.filter(r=>r.remainingPercent!==null&&r.remainingPercent<=10).length},quotaColumns:quotaColumns(rows.map(r=>r.quota)),accounts:rows};
}
export function buildContextGauge({usedTokens,maxTokens,checkpointIndex=0,threshold=0.8}={}){
  const valid=Number.isFinite(usedTokens)&&usedTokens>=0&&Number.isFinite(maxTokens)&&maxTokens>0;
  const percentage=valid?Math.min(100,Math.max(0,100*usedTokens/maxTokens)):null;
  const risk=percentage===null?'unknown':percentage>=threshold*100?'high':percentage>=threshold*80?'watch':'normal';
  return {percentage,risk,compressed:Number.isSafeInteger(checkpointIndex)&&checkpointIndex>0};
}
export function dailyUsageBuckets(points){
  if(!Array.isArray(points))throw new TypeError('points required');
  const arr=points.map(p=>({...p,ms:Date.parse(p.at)})).filter(p=>Number.isFinite(p.ms)&&Number.isFinite(p.remainingPercent)&&p.remainingPercent>=0&&p.remainingPercent<=100).sort((a,b)=>a.ms-b.ms);
  const byDay=new Map();
  for(let i=1;i<arr.length;i++){
    const prev=arr[i-1],now=arr[i];if(prev.ms===now.ms)continue;
    const day=new Date(now.ms).toISOString().slice(0,10);
    if(new Date(prev.ms).toISOString().slice(0,10)!==day)continue; // Daily attribution is unknown across midnight.
    const current=byDay.get(day)??{day,usedPercentagePoints:0,resets:0};
    const change=prev.remainingPercent-now.remainingPercent;
    if(change>0)current.usedPercentagePoints+=change;
    if(change<0)current.resets++;
    byDay.set(day,current);
  }
  return [...byDay.values()].sort((a,b)=>a.day.localeCompare(b.day));
}
