/**
 * Credential-free adapters for the quota data shapes used by Antigravity
 * Manager and Cockpit Tools. The host owns OAuth, transport and cache writes.
 * Source provenance: manager/src-tauri/src/models/quota.rs and
 * cockpit/crates/cockpit-core/src/models/quota.rs.
 */
const SOURCES = new Set(['antigravity-manager', 'cockpit-tools']);
const TOKEN_ID = /^[\p{L}\p{N}_.:/-]{1,256}$/u;
const MAX_WINDOWS = 512;
const DEFAULT_MAX_AGE_MS = 15 * 60 * 1000;

function identity(value, label) {
  if (typeof value !== 'string' || !TOKEN_ID.test(value)) {
    throw new TypeError(`${label} must be a nonempty scoped identifier`);
  }
  return value;
}
function resetEpoch(value, observedAt) {
  if (typeof value !== 'string' || !/^\d{4}-\d{2}-\d{2}T/.test(value)) return null;
  const epoch = Date.parse(value);
  return Number.isFinite(epoch) && epoch > observedAt ? epoch : null;
}
function boundedPercent(value) {
  return typeof value === 'number' && Number.isFinite(value) && value >= 0 && value <= 100 ? value : null;
}
function tierFromAuthoritativeFields(payload) {
  const values = [payload.tier_id, payload.subscription_tier];
  const known = new Set();
  for (const value of values) {
    if (typeof value !== 'string' || !value.trim()) continue;
    const v = value.trim().toLowerCase();
    if (['ultra','g1-ultra-tier','google ai ultra','google_one_helium'].includes(v)) known.add('ULTRA');
    else if (['pro','g1-pro-tier','google ai pro','standard-tier','premium'].includes(v)) known.add('PRO');
    else if (['free','free-tier','antigravity starter quota'].includes(v)) known.add('FREE');
  }
  return known.size === 1 ? [...known][0] : null;
}

/** Convert trusted, already-fetched peer quota data to an allowlisted UI DTO. */
export function normalizeAccountQuota({provider,accountRef,origin,payload,now=Date.now(),maxAgeMs=DEFAULT_MAX_AGE_MS}) {
  identity(provider,'provider');
  identity(accountRef,'accountRef');
  if (!SOURCES.has(origin)) throw new TypeError('unsupported quota source');
  if (!payload || typeof payload !== 'object' || Array.isArray(payload)) throw new TypeError('quota payload must be an object');
  const observedAt = Number(payload.last_updated) * 1000;
  if (!Number.isSafeInteger(observedAt) || observedAt <= 0 || observedAt > now + 5*60*1000) {
    throw new TypeError('quota observation timestamp is invalid');
  }
  if (!Number.isFinite(maxAgeMs) || maxAgeMs < 0) throw new TypeError('invalid freshness budget');
  const freshness = now-observedAt <= maxAgeMs ? 'fresh' : 'stale';
  const windows = new Map();
  const append = (id,remainingPercent,resetValue) => {
    if (windows.size>=MAX_WINDOWS || !TOKEN_ID.test(id.slice(id.indexOf(':')+1))) return;
    const pct=boundedPercent(remainingPercent), resetAt=resetEpoch(resetValue,observedAt);
    if (pct===null || resetAt===null) return;
    const existing=windows.get(id);
    if (!existing || resetAt<existing.resetAt) windows.set(id,{id,remainingPercent:pct,resetAt});
  };
  for (const model of Array.isArray(payload.models) ? payload.models : []) {
    if (model && typeof model.name === 'string') append(`model:${model.name}`,model.percentage,model.reset_time);
  }
  for (const group of Array.isArray(payload.quota_groups) ? payload.quota_groups : []) {
    for (const bucket of Array.isArray(group?.buckets) ? group.buckets : []) {
      if (!bucket || typeof bucket.bucket_id !== 'string') continue;
      const fraction=bucket.remaining_fraction;
      if (typeof fraction === 'number' && Number.isFinite(fraction) && fraction>=0 && fraction<=1) {
        append(`bucket:${bucket.bucket_id}`,Math.round(fraction*10000)/100,bucket.reset_time);
      }
    }
  }
  const forbidden=payload.is_forbidden===true;
  const list=[...windows.values()].sort((a,b)=>a.id.localeCompare(b.id));
  const state=forbidden?'forbidden':freshness==='stale'||!list.length?'unknown':list.some(x=>x.remainingPercent===0)?'exhausted':'available';
  return {provider,accountRef,origin,observedAt,freshness,state,tier:tierFromAuthoritativeFields(payload),windows:list};
}

/** Empty or out-of-order successes must never roll back a verified observation. */
export function mergeQuotaSnapshots(previous,next) {
  if (!previous) return next;
  if (!next) return previous;
  if (previous.provider !== next.provider || previous.accountRef !== next.accountRef) throw new Error('quota scope mismatch');
  if (next.observedAt < previous.observedAt ||
      (next.observedAt === previous.observedAt && previous.windows.length>=next.windows.length) ||
      (next.windows.length===0 && next.state!=='forbidden')) return previous;
  return next;
}

/** Read-only summary; stale and forbidden observations never imply capacity. */
export function summarizeQuotaWindows(snapshot,now=Date.now()) {
  const current=snapshot?.windows?.filter(w=>Number.isFinite(w.resetAt)&&w.resetAt>now)??[];
  const usable=snapshot?.freshness==='fresh' && snapshot?.state!=='forbidden' && current.length>0;
  return {
    windowCount:current.length,
    earliestResetAt:current.length?Math.min(...current.map(x=>x.resetAt)):null,
    lowestRemainingPercent:usable?Math.min(...current.map(x=>x.remainingPercent)):null,
    status:!usable?'unknown':snapshot.state,
  };
}
