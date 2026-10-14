/** In-process advisory direct-key routing; never owns a delegated gateway's credentials. */
import {safeId} from './catalog.mjs';
function assertOwner(owner){if(owner!=='host'&&owner!=='prodex')throw new TypeError('unrecognized routing owner')}
const isPct=x=>typeof x==='number'&&Number.isFinite(x)&&x>=0&&x<=100;
const clean=(x)=>({
  id:x.id,provider:x.provider,models:Array.isArray(x.models)?x.models:[],
  health:x.health??'unknown',remainingPercent:isPct(x.remainingPercent)?x.remainingPercent:null,
  quotaFetchedAt:Number.isFinite(x.quotaFetchedAt)?x.quotaFetchedAt:null,
  cooldownUntil:Number.isFinite(x.cooldownUntil)?x.cooldownUntil:0,
  inFlight:Number.isSafeInteger(x.inFlight)&&x.inFlight>=0?x.inFlight:0,
  maxConcurrency:Number.isSafeInteger(x.maxConcurrency)&&x.maxConcurrency>0?x.maxConcurrency:4,
  retired:!!x.retired,
});
export function classifyProviderFailure(status,body=''){
  const s=typeof body==='string'?body.slice(0,1600).toLowerCase():'';
  if(status===429)return {kind:'rate-limited',cooldownSeconds:120};
  if(status===401)return {kind:'authentication',cooldownSeconds:120};
  if(status===403){
    if(s.includes('validation_required')||s.includes('verify your account')||s.includes('appeal_url'))return {kind:'verification-required',cooldownSeconds:600};
    return {kind:'transient-forbidden',cooldownSeconds:300};
  }
  if(Number.isInteger(status)&&status>=500)return {kind:'transient-upstream',cooldownSeconds:10};
  return {kind:'other',cooldownSeconds:0};
}
export function rankDirectAccounts(accounts,{provider,model,now=Date.now()/1000,routeOwner='host',preferredAccountId=null}={}){
  assertOwner(routeOwner);
  if(routeOwner!=='host')return [];
  return accounts.map(x=>{const a=clean(x);if(!safeId(a.id,128)||!safeId(a.provider,128)||a.models.some(m=>!safeId(m)))throw new TypeError('invalid account metadata');return a}).filter(x=>
    x.provider===provider&&x.models.includes(model)&&!x.retired&&
    (x.health==='healthy'||x.health==='refreshed')&&isPct(x.remainingPercent)&&x.remainingPercent>0&&
    x.quotaFetchedAt!==null&&x.quotaFetchedAt<=now+60&&now-x.quotaFetchedAt<3600&&
    x.cooldownUntil<=now&&x.inFlight<x.maxConcurrency
  ).sort((a,b)=>{
    const preferred=Number(b.id===preferredAccountId)-Number(a.id===preferredAccountId);
    if(preferred)return preferred;
    // Remaining quota wins when the account is idle; high concurrent load reduces its score.
    const sa=a.remainingPercent/(1+a.inFlight),sb=b.remainingPercent/(1+b.inFlight);
    return sb-sa||a.id.localeCompare(b.id);
  });
}
export class FleetController{
  #accounts=new Map();#leases=new Map();#completed=new Map();#sessions=new Map();#now;
  constructor(accounts=[],{now=()=>Date.now()/1000}={}){if(typeof now!=='function')throw new TypeError('now must be a function');this.#now=now;this.updateAccounts(accounts)}
  updateAccounts(accounts){
    if(!Array.isArray(accounts)||accounts.length>10000)throw new TypeError('invalid accounts');
    const next=new Map();
    for(const input of accounts){
      const a=clean(input);
      if(!safeId(a.id,128)||!safeId(a.provider,128)||next.has(a.id)||a.models.some(m=>!safeId(m)))throw new TypeError('invalid or duplicate account metadata');
      const previous=this.#accounts.get(a.id);
      if(previous&&previous.provider!==a.provider)throw new TypeError('account identity cannot change providers');
      if(previous){a.inFlight=previous.inFlight;a.cooldownUntil=Math.max(a.cooldownUntil,previous.cooldownUntil)}
      next.set(a.id,a);
    }
    for(const [id,a] of this.#accounts){if(!next.has(id)&&a.inFlight>0)next.set(id,{...a,retired:true});}
    this.#accounts=next;
  }
  acquire({provider,model,requestId,sessionKey,routeOwner='host'}){
    assertOwner(routeOwner);
    if(routeOwner!=='host')return {delegated:true};
    if(typeof requestId!=='string'||requestId.length===0||requestId.length>200)throw new TypeError('invalid request id');
    if(this.#completed.has(requestId))throw new Error('completed request id cannot be reused');
    if(this.#leases.has(requestId)){
      const prior=this.#leases.get(requestId);
      if(prior.provider!==provider||prior.model!==model||prior.sessionKey!==sessionKey)throw new Error('request id collision');
      return {...prior.result};
    }
    const affinity=sessionKey?this.#sessions.get(JSON.stringify([provider,model,sessionKey])):null;
    const ranked=rankDirectAccounts([...this.#accounts.values()],{provider,model,now:this.#now(),preferredAccountId:affinity});
    if(!ranked.length)return {unavailable:true,reason:'no-eligible-account'};
    const account=this.#accounts.get(ranked[0].id);
    account.inFlight++;
    const result={accountId:account.id,requestId};
    this.#leases.set(requestId,{result,provider,model,sessionKey,accountId:account.id});
    return {...result};
  }
  release(requestId,{success=false,status=null,body='',committed=false}={}){
    if(this.#completed.has(requestId))return {...this.#completed.get(requestId)};
    const lease=this.#leases.get(requestId);
    if(!lease)return {unknownRequest:true,retryable:false};
    this.#leases.delete(requestId);
    const account=this.#accounts.get(lease.accountId);
    if(account){account.inFlight=Math.max(0,account.inFlight-1);if(status){
      const classification=classifyProviderFailure(status,body);
      account.cooldownUntil=Math.max(account.cooldownUntil,this.#now()+classification.cooldownSeconds);
      if(classification.kind==='authentication')account.health='needs-refresh';
      if(classification.kind==='verification-required')account.health='verification-required';
    }}
    if(success&&lease.sessionKey)this.#sessions.set(JSON.stringify([lease.provider,lease.model,lease.sessionKey]),lease.accountId);
    const retryable=!committed&&!success&&status!==401&&status!==403&&(status===429||(Number.isInteger(status)&&status>=500));
    const result={success:!!success,retryable};
    this.#completed.set(requestId,result);
    if(account?.retired&&account.inFlight===0)this.#accounts.delete(account.id);
    return {...result};
  }
  snapshot(){return {accounts:[...this.#accounts.values()].map(a=>({...a,models:[...a.models]})).sort((a,b)=>a.id.localeCompare(b.id)),inFlight:this.#leases.size,completed:this.#completed.size};}
}
