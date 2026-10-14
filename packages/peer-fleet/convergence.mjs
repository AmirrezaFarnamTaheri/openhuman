/** Second-order adapter: one catalog, one read model, and exactly one routing owner. */
import {buildCatalog} from './catalog.mjs';
import {mergeQuota,normalizeQuota,quotaRemaining} from './quota.mjs';
import {buildDashboard} from './widgets.mjs';
import {rankDirectAccounts} from './routing.mjs';
import {normalizePreset,resolvePresetGroup} from './presets.mjs';

function publicAccount(a){
  return {id:a.id,provider:a.provider,models:Array.isArray(a.models)?[...a.models]:[],health:a.health??'unknown',healthObservedAt:Number.isFinite(a.healthObservedAt)?a.healthObservedAt:null,quota:a.quota??null};
}
export function reconcileLiveObservations(current=[],incoming=[]){
  if(!Array.isArray(current)||!Array.isArray(incoming))throw new TypeError('invalid observations');
  const prior=new Map(current.map(a=>[a.id,publicAccount(a)]));
  if(prior.size!==current.length)throw new TypeError('duplicate current account identity');
  const merged=new Map(prior);const seen=new Set();
  for(const received of incoming){
    const a=publicAccount(received),old=prior.get(a.id);
    if(seen.has(a.id))throw new TypeError('duplicate incoming account identity');
    seen.add(a.id);
    if(old&&old.provider!==a.provider)throw new TypeError('account provider identity changed');
    if(old){
      const healthNewest=a.healthObservedAt!==null && (old.healthObservedAt===null||a.healthObservedAt>=old.healthObservedAt);
      merged.set(a.id,{...a,health:healthNewest?a.health:old.health,healthObservedAt:healthNewest?a.healthObservedAt:old.healthObservedAt,quota:mergeQuota(old.quota,a.quota)});
    }else merged.set(a.id,a);
  }
  return [...merged.values()].sort((a,b)=>a.id.localeCompare(b.id));
}
export function recommendModelRoute({accounts=[],provider,model,now=Date.now()/1000,routeOwner='host'}={}){
  if(routeOwner!=='host')return {mode:'delegated',accountId:null,owner:routeOwner};
  const metadata=accounts.map(a=>{
    const q=normalizeQuota(a.quota??{fetchedAt:null,windows:[]},now);
    return {id:a.id,provider:a.provider,models:a.models,health:a.health,quotaFetchedAt:q.fetchedAt,remainingPercent:quotaRemaining(q),cooldownUntil:a.cooldownUntil,inFlight:a.inFlight,maxConcurrency:a.maxConcurrency};
  });
  const winner=rankDirectAccounts(metadata,{provider,model,now})[0];
  return {mode:'advisory',accountId:winner?.id??null,owner:'host'};
}
export function composeFleetView({accounts=[],declaredModels=[],liveModels=[],presets=[],groupId=null,routeOwner='host',now=Date.now()/1000}={}){
  const safeAccounts=accounts.map(publicAccount);
  const catalog=buildCatalog(declaredModels,liveModels);
  const dashboard=buildDashboard(safeAccounts,{now});
  const normalizedPresets=presets.map(normalizePreset);
  const selected=groupId?resolvePresetGroup(normalizedPresets,groupId):[];
  const selectedView=selected.map(p=>({...p,liveModelCount:p.models.filter(id=>catalog.some(m=>m.id===id&&m.provider===p.provider&&m.availability==='live')).length}));
  const routed=routeOwner==='host'?safeAccounts.map(a=>{
    const normalized=normalizeQuota(a.quota??{fetchedAt:null,windows:[]},now);
    return {...a,remainingPercent:quotaRemaining(normalized),quotaFetchedAt:normalized.fetchedAt};
  }):[];
  const model=selected[0]?.models[0]??null;
  const provider=selected[0]?.provider??null;
  const advisory=routeOwner==='host'&&model?rankDirectAccounts(routed,{provider,model,now,routeOwner}):[];
  return {catalog,dashboard,groups:{selected:selectedView},routing:{owner:routeOwner,delegated:routeOwner!=='host',advisory:advisory.map(a=>({id:a.id,provider:a.provider,remainingPercent:a.remainingPercent}))}};
}
