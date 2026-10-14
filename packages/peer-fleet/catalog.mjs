/** Provider-scoped model metadata. Declarations are hints, never live provider facts. */
const INVISIBLE=/[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u;
const CAPABILITIES=['text','tools','vision','reasoning','imageOutput','audioInput','audioOutput','videoInput'];
export function safeId(value,max=256){return typeof value==='string'&&value.length>0&&value.length<=max&&!INVISIBLE.test(value)&&!(/\s/u).test(value)&&!/[<>&"'`{}\\]/u.test(value)}
function assertId(v,what,max){if(!safeId(v,max))throw new TypeError(`invalid ${what}`)}
function key(provider,id){return JSON.stringify([provider,id])}
function finitePositive(x){return Number.isSafeInteger(x)&&x>0}
function capability(v){return v===true?'supported':v===false?'unsupported':'unknown'}
const emptyCapabilities=()=>Object.fromEntries(CAPABILITIES.map(k=>[k,'unknown']));
export function buildCatalog(declarations=[],live=[]){
  if(!Array.isArray(declarations)||!Array.isArray(live)||declarations.length>10_000||live.length>10_000)throw new TypeError('invalid catalog bounds');
  const map=new Map();
  for(const entry of declarations){
    assertId(entry?.id,'declared model');assertId(entry?.provider,'declared provider',128);
    assertId(entry?.source,'peer source',160);
    const k=key(entry.provider,entry.id);
    const row=map.get(k)??{id:entry.id,provider:entry.provider,availability:'unverified',sources:[],conflicts:[],contextWindow:null,declaredContextWindow:null,capabilities:emptyCapabilities(),pricing:{known:false,inputPer1M:null,outputPer1M:null}};
    if(!row.sources.includes(entry.source))row.sources.push(entry.source);
    // Two conflicting offline assertions cannot establish a verified window.
    if(finitePositive(entry.contextWindow)&&!row.conflicts.includes('contextWindow')){
      if(row.declaredContextWindow===null)row.declaredContextWindow=entry.contextWindow;
      else if(row.declaredContextWindow!==entry.contextWindow){row.declaredContextWindow=null;row.conflicts.push('contextWindow')}
    }
    map.set(k,row);
  }
  const seenLive=new Set();
  for(const entry of live){
    assertId(entry?.id,'live model');assertId(entry?.provider,'live provider',128);
    const k=key(entry.provider,entry.id);
    if(seenLive.has(k))throw new TypeError('duplicate live provider-model key');
    seenLive.add(k);
    const row=map.get(k)??{id:entry.id,provider:entry.provider,availability:'unverified',sources:[],conflicts:[],contextWindow:null,declaredContextWindow:null,capabilities:emptyCapabilities(),pricing:{known:false,inputPer1M:null,outputPer1M:null}};
    row.availability='live';
    row.contextWindow=finitePositive(entry.contextWindow)?entry.contextWindow:null;
    row.capabilities=Object.fromEntries(CAPABILITIES.map(c=>[c,capability(entry.capabilities?.[c])]));
    if(entry.pricing?.known===true&&Number.isFinite(entry.pricing.inputPer1M)&&entry.pricing.inputPer1M>=0&&Number.isFinite(entry.pricing.outputPer1M)&&entry.pricing.outputPer1M>=0){
      row.pricing={known:true,inputPer1M:entry.pricing.inputPer1M,outputPer1M:entry.pricing.outputPer1M};
    }
    map.set(k,row);
  }
  return [...map.values()].map(r=>({...r,sources:r.sources.sort(),conflicts:r.conflicts.sort()})).sort((a,b)=>a.provider.localeCompare(b.provider)||a.id.localeCompare(b.id));
}
export function resolveModelAlias(provider,id,aliases=[]){
  assertId(provider,'provider',128);assertId(id,'model');
  if(!Array.isArray(aliases)||aliases.length>10_000)throw new TypeError('invalid alias set');
  const edges=new Map();
  for(const a of aliases){
    assertId(a.provider,'alias provider',128);assertId(a.alias,'alias');assertId(a.target,'alias target');
    if(a.provider!==provider)continue;
    if(edges.has(a.alias))throw new TypeError('duplicate or conflicting alias');
    edges.set(a.alias,a.target);
  }
  const visited=new Set();let current=id;
  while(edges.has(current)){
    if(visited.has(current))throw new TypeError('cyclic model aliases');
    visited.add(current);current=edges.get(current);
  }
  return current;
}
export function selectModelPreset(rows,provider,id){
  assertId(provider,'provider',128);assertId(id,'model');
  const row=rows.find(x=>x.provider===provider&&x.id===id);
  if(!row)return null;
  return {...row,imageOutput:{state:row.capabilities?.imageOutput??'unknown'}};
}
