/** Observation-based quota mathematics. No fabricated 100% when quota is missing. */
const DAY=86400;
const asNumber=x=>typeof x==='number'&&Number.isFinite(x)?x:null;
const validPct=x=>asNumber(x)!==null&&x>=0&&x<=100;
function durationKey(s){if(!(Number.isSafeInteger(s)&&s>0))return 'unknown';if(Math.abs(s-18000)<=300)return '5h';if(Math.abs(s-604800)<=3600)return 'weekly';if(s>=27*DAY&&s<=32*DAY)return 'monthly';return `duration:${s}`}
function normalizedWindow(w,fetchedAt,now){
  const duration=Number.isSafeInteger(w?.durationSeconds)&&w.durationSeconds>0?w.durationSeconds:null;
  let used=validPct(w?.usedPercent)?w.usedPercent:null;
  const remaining=validPct(w?.remainingPercent)?w.remainingPercent:null;
  const contradiction=used!==null&&remaining!==null&&Math.abs(used+remaining-100)>0.01;
  const badValue=(w?.usedPercent!=null&&!validPct(w.usedPercent))||(w?.remainingPercent!=null&&!validPct(w.remainingPercent));
  if(used===null&&remaining!==null)used=100-remaining;
  const reset=asNumber(w?.resetAt);
  const stale=reset!==null&&reset<=now&&reset>fetchedAt;
  const freshness=badValue||contradiction?'invalid':stale?'stale':used===null?'unknown':'fresh';
  return {key:durationKey(duration),durationSeconds:duration,resetAt:reset,usedPercent:freshness==='invalid'?null:used,remainingPercent:freshness==='invalid'||used===null?null:Math.max(0,100-used),freshness};
}
export function normalizeQuota(input,now=Date.now()/1000,{maxAgeSeconds=3600}={}){
  const fetchedAt=asNumber(input?.fetchedAt);
  const validTime=fetchedAt!==null&&fetchedAt>0&&fetchedAt<=now+60;
  if(input?.windows!=null&&!Array.isArray(input.windows))throw new TypeError('quota windows must be an array');
  if((input?.windows?.length??0)>100)throw new TypeError('too many quota windows');
  const windows=(input?.windows??[]).map(w=>normalizedWindow(w,fetchedAt??0,now));
  const freshness=!validTime?'invalid':now-fetchedAt>=maxAgeSeconds?'stale':windows.some(w=>w.freshness==='invalid')?'invalid':windows.some(w=>w.freshness==='stale')?'stale':windows.some(w=>w.freshness==='fresh')?'fresh':'unknown';
  return {fetchedAt,windows,freshness};
}
export function mergeQuota(current,incoming){
  if(!current)return incoming??null;
  if(!incoming)return current;
  const a=asNumber(current.fetchedAt),b=asNumber(incoming.fetchedAt);
  if(b===null)return current;
  if(a===null)return incoming;
  return b>a?incoming:current;
}
export function quotaRemaining(normalized){
  if(!normalized||normalized.freshness==='invalid'||normalized.freshness==='stale')return null;
  const values=normalized.windows.filter(w=>w.freshness==='fresh'&&validPct(w.remainingPercent)).map(w=>w.remainingPercent);
  return values.length?Math.min(...values):null;
}
export function quotaColumns(quotaSnapshots){
  const seen=new Map();
  for(const snap of quotaSnapshots){for(const w of snap?.windows??[]){
    if(w.remainingPercent===null||w.durationSeconds===null)continue;
    seen.set(w.key,{key:w.key,durationSeconds:w.durationSeconds});
  }}
  return [...seen.values()].sort((a,b)=>a.durationSeconds-b.durationSeconds||a.key.localeCompare(b.key));
}
export function estimateRunway(points){
  if(!Array.isArray(points)||points.length<2)return null;
  const sorted=[...points].sort((a,b)=>a.at-b.at);
  const a=sorted.at(-2),b=sorted.at(-1);
  if(!Number.isFinite(a.at)||!Number.isFinite(b.at)||!validPct(a.remainingPercent)||!validPct(b.remainingPercent))return null;
  const delta=b.at-a.at; if(delta<=0||delta>DAY)return null;
  const used=a.remainingPercent-b.remainingPercent;
  if(used<=0)return null; // a reset or stable data is not a positive usage rate.
  const perHour=used*3600/delta;
  if(!Number.isFinite(perHour)||perHour<=0)return null;
  return {usagePointsPerHour:perHour,hours:b.remainingPercent/perHour,confidence:'two-point-estimate'};
}
