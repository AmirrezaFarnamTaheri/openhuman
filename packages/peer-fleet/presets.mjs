/** Credential-free, revisioned provider/model presets. Secrets remain in host secure storage. */
import {safeId} from './catalog.mjs';
const FIELDS=new Set(['id','name','provider','baseUrl','models','revision','groupId','deleted','enabled','updatedAt','keyRef']);
const isLabel=x=>typeof x==='string'&&x.length>0&&x.length<=180&&!/[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u.test(x);
function normalizeUrl(value){
  if(typeof value!=='string')throw new TypeError('missing endpoint');
  let url;try{url=new URL(value)}catch{throw new TypeError('invalid endpoint URL')}
  const local=['localhost','127.0.0.1','[::1]'].includes(url.hostname);
  if(url.protocol!=='https:'&&!(url.protocol==='http:'&&local))throw new TypeError('remote HTTPS is required');
  if(url.username||url.password||url.search||url.hash)throw new TypeError('endpoint cannot embed credentials or query fragments');
  if(url.pathname.length>512)throw new TypeError('endpoint path too long');
  if(/%(?:0[0-9a-f]|1[0-9a-f]|7f)/i.test(url.pathname))throw new TypeError('endpoint contains encoded controls');
  return url.href.replace(/\/$/,'');
}
export function normalizePreset(input){
  if(!input||typeof input!=='object'||Array.isArray(input))throw new TypeError('invalid preset');
  for(const k of Object.keys(input))if(!FIELDS.has(k))throw new TypeError('unknown or credential-bearing preset key');
  if(!safeId(input.id,128)||!safeId(input.provider,128)||!isLabel(input.name))throw new TypeError('invalid preset identity');
  if(!Number.isSafeInteger(input.revision)||input.revision<0)throw new TypeError('invalid preset revision');
  if(!Array.isArray(input.models)||input.models.length>1000||input.models.some(x=>!safeId(x)))throw new TypeError('invalid model list');
  if(input.groupId!=null&&!safeId(input.groupId,128))throw new TypeError('invalid group id');
  if(input.keyRef!=null&&!safeId(input.keyRef,128))throw new TypeError('invalid secure-key reference');
  const models=[...new Set(input.models)];
  return {id:input.id,name:input.name,provider:input.provider,baseUrl:normalizeUrl(input.baseUrl),models,revision:input.revision,groupId:input.groupId??null,deleted:input.deleted===true,enabled:input.enabled!==false,updatedAt:Number.isFinite(input.updatedAt)?input.updatedAt:null,keyRef:input.keyRef??null};
}
function equal(a,b){return JSON.stringify(a)===JSON.stringify(b)}
export function mergePresetSync(local,remote){
  if(!Array.isArray(local)||!Array.isArray(remote))throw new TypeError('invalid preset collections');
  const byId=new Map();const conflicts=[];
  for(const input of local){const p=normalizePreset(input);if(byId.has(p.id))throw new TypeError('duplicate local preset');byId.set(p.id,p)}
  const incoming=new Set();
  for(const input of remote){
    const p=normalizePreset(input);if(incoming.has(p.id))throw new TypeError('duplicate remote preset');incoming.add(p.id);
    const existing=byId.get(p.id);
    if(!existing||p.revision>existing.revision)byId.set(p.id,p);
    else if(p.revision===existing.revision&&!equal(p,existing))conflicts.push({id:p.id,reason:'concurrent-edit'});
  }
  return {merged:[...byId.values()].sort((a,b)=>a.id.localeCompare(b.id)),conflicts};
}
export function applyPresetBatch(presets,edits){
  if(!Array.isArray(presets)||!Array.isArray(edits))throw new TypeError('invalid batch');
  const map=new Map(presets.map(input=>{const p=normalizePreset(input);return [p.id,p]}));
  if(map.size!==presets.length)throw new TypeError('duplicate presets');
  const seen=new Set();
  for(const edit of edits){
    if(!edit||!safeId(edit.id,128)||seen.has(edit.id))throw new TypeError('duplicate/invalid batch edit');seen.add(edit.id);
    const item=map.get(edit.id);if(!item)throw new Error('preset not found');
    if(item.revision!==edit.expectedRevision)throw new Error('stale preset revision');
    if(!edit.changes||typeof edit.changes!=='object'||'revision' in edit.changes||'id' in edit.changes)throw new Error('invalid preset edit');
    map.set(edit.id,normalizePreset({...item,...edit.changes,revision:item.revision+1}));
  }
  return [...map.values()].sort((a,b)=>a.id.localeCompare(b.id));
}
export function resolvePresetGroup(presets,groupId){
  if(!safeId(groupId,128))throw new TypeError('invalid group');
  return presets.map(normalizePreset).filter(p=>p.groupId===groupId&&!p.deleted&&p.enabled).sort((a,b)=>a.id.localeCompare(b.id));
}
