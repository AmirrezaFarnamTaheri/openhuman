/** Single-flight refresh, with a single coalesced replay and bounded scheduler hints. */
export function createCoalescedRefresh(fn){
  if(typeof fn!=='function')throw new TypeError('refresh must be callable');
  let flight=null;let requested=false;
  return ()=>{
    if(flight){requested=true;return flight}
    requested=true;
    flight=Promise.resolve().then(async()=>{do{requested=false;await fn()}while(requested)}).finally(()=>{flight=null});
    return flight;
  };
}
export function nextRefreshDue(account,now=Date.now()/1000){
  if(!account||typeof account!=='object')throw new TypeError('invalid account');
  const active=account.active===true;
  const remaining=account.remainingPercent;
  const low=Number.isFinite(remaining)&&remaining>=0&&remaining<=15;
  const base=active?(low?300:900):(low?900:3600);
  const failures=Number.isSafeInteger(account.failures)?Math.max(0,Math.min(20,account.failures)):0;
  const backoff=failures>0?Math.min(21600,60*2**(failures-1)):0;
  const interval=Math.max(base,backoff);
  if(failures>0&&!Number.isFinite(account.lastAttemptAt))return now; // Missing attempt time: refresh immediately, not a sliding deadline.
  const last=failures>0
    ? account.lastAttemptAt
    : (Number.isFinite(account.lastFetchedAt)?account.lastFetchedAt:now-base);
  return Math.max(0,last+interval);
}
