import test from 'node:test';
import assert from 'node:assert/strict';
import { normalizeAccountQuota, mergeQuotaSnapshots, summarizeQuotaWindows } from '../observations.mjs';

const now = Date.parse('2026-10-14T12:00:00Z');
const baseline = {
  last_updated: now / 1000 - 20,
  subscription_tier: 'g1-ultra-tier',
  models: [{name: 'gemini-3-flash', percentage: 75, reset_time: '2026-10-15T12:00:00Z'}],
  quota_groups: [{display_name: 'Gemini', buckets: [
    {bucket_id: 'gemini-weekly', window: 'weekly', remaining_fraction: .5, reset_time: '2026-10-21T00:00:00Z'},
    {bucket_id: 'gemini-5h', window: '5h', remaining_fraction: .2, reset_time: '2026-10-14T15:00:00Z'},
  ]}],
};

test('normalizes manager quota into independent model and grouped windows', () => {
  const q = normalizeAccountQuota({provider:'antigravity', accountRef:'opaque-ref', origin:'antigravity-manager', payload:baseline, now});
  assert.equal(q.tier, 'ULTRA');
  assert.equal(q.freshness,'fresh');
  assert.equal(q.windows.length,3);
  assert.equal(q.windows.find(x=>x.id==='bucket:gemini-5h').remainingPercent,20);
  assert.equal(q.windows.find(x=>x.id==='model:gemini-3-flash').remainingPercent,75);
  assert.equal(q.accountRef,'opaque-ref');
  assert.equal(JSON.stringify(q).includes('email'),false);
});

test('cockpit tiers must come from explicit authoritative IDs and must not be inferred from model names', () => {
  const q = normalizeAccountQuota({provider:'antigravity', accountRef:'account-1', origin:'cockpit-tools', now,payload:{last_updated:now/1000,models:[{name:'claude-opus',percentage:100,reset_time:'2026-10-15T12:00:00Z'}],email:'private@example.com', access_token:'secret'}});
  assert.equal(q.tier,null);
  assert.equal(q.windows.length,1);
  assert(!JSON.stringify(q).includes('private@example.com'));
  assert(!JSON.stringify(q).includes('secret'));
});

test('forbidden and stale snapshots cannot be misrepresented as healthy', () => {
  const forbidden=normalizeAccountQuota({provider:'antigravity',accountRef:'opaque',origin:'cockpit-tools',now,payload:{...baseline,is_forbidden:true}});
  assert.equal(forbidden.state,'forbidden');
  const stale=normalizeAccountQuota({provider:'antigravity',accountRef:'opaque',origin:'cockpit-tools',now,payload:{...baseline,last_updated:now/1000 - 7200}});
  assert.equal(stale.freshness,'stale');
  assert.equal(stale.state,'unknown');
});

test('rejects invalid percentages, fractions and reset boundaries instead of inventing quota', () => {
  const quota=normalizeAccountQuota({provider:'antigravity',accountRef:'one',origin:'antigravity-manager',now,payload:{...baseline,models:[{name:'bad',percentage:999,reset_time:'invalid'}, {name:'valid',percentage:0,reset_time:'2026-10-15T12:00:00Z'}],quota_groups:[{buckets:[{bucket_id:'bad',window:'weekly',remaining_fraction:NaN,reset_time:'2026-10-15T12:00:00Z'}]}]}});
  assert.deepEqual(quota.windows.map(w=>w.id),['model:valid']);
  assert.equal(quota.state,'exhausted');
});

test('new empty model payload never overwrites older verified windows', () => {
  const old=normalizeAccountQuota({provider:'antigravity',accountRef:'same',origin:'cockpit-tools',now,payload:baseline});
  const next=normalizeAccountQuota({provider:'antigravity',accountRef:'same',origin:'cockpit-tools',now,payload:{last_updated:now/1000,models:[]}});
  const merged=mergeQuotaSnapshots(old,next);
  assert.equal(merged.windows.length,3);
  assert.equal(merged.observedAt,old.observedAt);
});

test('older observations cannot rewind newer verified state', () => {
  const a=normalizeAccountQuota({provider:'antigravity',accountRef:'same',origin:'cockpit-tools',now,payload:baseline});
  const b=normalizeAccountQuota({provider:'antigravity',accountRef:'same',origin:'cockpit-tools',now,payload:{...baseline,last_updated:now/1000-60}});
  assert.equal(mergeQuotaSnapshots(a,b),a);
});

test('multi-window recommendations constrain availability to the worst fresh bucket, without collapsing windows', () => {
  const q=normalizeAccountQuota({provider:'antigravity',accountRef:'same',origin:'antigravity-manager',now,payload:baseline});
  const x=summarizeQuotaWindows(q,now);
  assert.equal(x.lowestRemainingPercent,20);
  assert.equal(x.earliestResetAt,Date.parse('2026-10-14T15:00:00Z'));
  assert.equal(x.windowCount,3);
});

test('invalid identity and nonpositive observation timestamps are rejected',()=>{
  assert.throws(()=>normalizeAccountQuota({provider:'antigravity',accountRef:'',origin:'cockpit-tools',now,payload:baseline}));
  assert.throws(()=>normalizeAccountQuota({provider:'antigravity',accountRef:'same',origin:'cockpit-tools',now,payload:{...baseline,last_updated:0}}));
});

test('partial later refresh merges independent model windows without losing unmentioned live buckets',()=>{
 const previous=normalizeAccountQuota({provider:'antigravity',accountRef:'id',origin:'antigravity-manager',now,payload:baseline});
 const later=normalizeAccountQuota({provider:'antigravity',accountRef:'id',origin:'antigravity-manager',now,payload:{last_updated:now/1000-10,models:[{name:'gemini-3-flash',percentage:40,reset_time:'2026-10-15T12:00:00Z'}]}});
 const merged=mergeQuotaSnapshots(previous,later);
 assert.equal(merged.windows.length,3);
 assert.equal(merged.windows.find(w=>w.id==='model:gemini-3-flash').remainingPercent,40);
 assert.equal(merged.windows.find(w=>w.id==='bucket:gemini-weekly').remainingPercent,50);
});

test('a forbidden observation supersedes cached availability even at an equal timestamp',()=>{
 const previous=normalizeAccountQuota({provider:'antigravity',accountRef:'id',origin:'cockpit-tools',now,payload:baseline});
 const blocked=normalizeAccountQuota({provider:'antigravity',accountRef:'id',origin:'cockpit-tools',now,payload:{...baseline,is_forbidden:true,models:[],quota_groups:[]}});
 assert.equal(mergeQuotaSnapshots(previous,blocked).state,'forbidden');
});
