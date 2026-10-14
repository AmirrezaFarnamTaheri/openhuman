import {test} from 'node:test';
import assert from 'node:assert/strict';
import {normalizeQuota, mergeQuota, estimateRunway, quotaColumns, quotaRemaining} from '../quota.mjs';
const NOW=1_800_000_000;
const quota=(fetchedAt, usedPercent=20, other={})=>({fetchedAt,windows:[{durationSeconds:18000,usedPercent,resetAt:NOW+3600,...other}]});
test('quota normalizes a healthy five-hour window with remaining and time',()=>{
  const a=normalizeQuota(quota(NOW-60),NOW);
  assert.equal(a.freshness,'fresh');
  assert.equal(a.windows[0].remainingPercent,80);
  assert.equal(a.windows[0].key,'5h');
});
test('quota flags expired, future and invalid reports without inventing remaining',()=>{
  assert.equal(normalizeQuota(quota(NOW-3900),NOW).freshness,'stale');
  assert.equal(normalizeQuota(quota(NOW+120),NOW).freshness,'invalid');
  assert.equal(normalizeQuota(quota(NOW-50,120),NOW).windows[0].remainingPercent,null);
  assert.equal(normalizeQuota({fetchedAt:NOW,windows:[{durationSeconds:604800}]},NOW).windows[0].remainingPercent,null);
});
test('passed reset dates invalidate a previously fetched quota sample',()=>{
  const a=normalizeQuota(quota(NOW-20,40,{resetAt:NOW-1}),NOW);
  assert.equal(a.windows[0].freshness,'stale');
});
test('older refresh responses cannot overwrite newer account quota state',()=>{
  const a=quota(NOW-30,90),b=quota(NOW-60,5);
  assert.deepEqual(mergeQuota(a,b),a);
  assert.deepEqual(mergeQuota(b,a),a);
});
test('equal timestamp conflicts preserve current observation for determinism',()=>{
  assert.deepEqual(mergeQuota(quota(NOW,9),quota(NOW,5)),quota(NOW,9));
});
test('runway does not interpret reset rebound as negative consumption',()=>{
  assert.equal(estimateRunway([{at:NOW-60*60,remainingPercent:2},{at:NOW,remainingPercent:99}]),null);
});
test('runway forecasts only when monotonic consumption is observed',()=>{
  const a=estimateRunway([{at:NOW-3600,remainingPercent:70},{at:NOW,remainingPercent:60}]);
  assert.equal(a.usagePointsPerHour,10);
  assert.equal(a.hours,6);
});
test('quota columns adapt to account-specific windows without duplicates',()=>{
  const q1=normalizeQuota(quota(NOW),NOW);
  const q2=normalizeQuota({fetchedAt:NOW,windows:[{durationSeconds:604800,usedPercent:12}]},NOW);
  assert.deepEqual(quotaColumns([q2,q1,q1]).map(x=>x.key),['5h','weekly']);
});
test('quotaRemaining uses tightest available window and never assumes unknown is full',()=>{
  const q=normalizeQuota({fetchedAt:NOW,windows:[{durationSeconds:18000,usedPercent:30},{durationSeconds:604800,usedPercent:95}]},NOW);
  assert.equal(quotaRemaining(q),5);
  assert.equal(quotaRemaining(normalizeQuota({fetchedAt:NOW,windows:[]},NOW)),null);
});
