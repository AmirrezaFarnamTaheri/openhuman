import {test} from 'node:test';
import assert from 'node:assert/strict';
import {createCoalescedRefresh,nextRefreshDue} from '../refresh.mjs';
import {buildDashboard,buildContextGauge,dailyUsageBuckets} from '../widgets.mjs';
const NOW=1_800_000_000;
test('coalesced refresh replays only once for burst requests',async()=>{
  let runs=0;let resolveFirst;
  const run=createCoalescedRefresh(async()=>{runs++;if(runs===1)await new Promise(resolve=>resolveFirst=resolve);});
  const promise=run();await Promise.resolve();await Promise.resolve();
  const p2=run();run();resolveFirst();await Promise.all([promise,p2]);
  assert.equal(runs,2);
});
test('coalesced refresh resets single flight after rejected refresh',async()=>{
  let n=0;const run=createCoalescedRefresh(async()=>{if(++n===1)throw new Error('failed')});
  await assert.rejects(run());await run();assert.equal(n,2);
});
test('refresh plan honors exponential account backoff and reset proximity',()=>{
  const common={id:'a',lastFetchedAt:NOW-4000,failures:0,active:true};
  assert.ok(nextRefreshDue(common,NOW)<=NOW);
  assert.ok(nextRefreshDue({...common,failures:7,lastAttemptAt:NOW-100},NOW)>NOW);
  assert.ok(nextRefreshDue({...common,active:false,failures:0},NOW)>nextRefreshDue(common,NOW));
});
test('dashboard provides quota cells and distinguishes unknown from 0% remaining',()=>{
  const rows=buildDashboard([{id:'a',provider:'google',health:'healthy',quota:{fetchedAt:NOW,windows:[{durationSeconds:18000,usedPercent:100}]}},{id:'b',provider:'google',health:'unknown'}],{now:NOW});
  assert.equal(rows.summary.total,2);assert.equal(rows.accounts[0].remainingPercent,0);assert.equal(rows.accounts[1].remainingPercent,null);
});
test('context gauge handles invalid and compressed context without overpromising',()=>{
  assert.equal(buildContextGauge({usedTokens:100,maxTokens:0}).percentage,null);
  assert.equal(buildContextGauge({usedTokens:900,maxTokens:1000,checkpointIndex:2}).risk,'high');
});
test('usage daily buckets distinguish reset increases from actual usage',()=>{
  const points=[{at:'2026-10-13T10:00:00Z',remainingPercent:70},{at:'2026-10-13T11:00:00Z',remainingPercent:60},{at:'2026-10-13T12:00:00Z',remainingPercent:90}];
  assert.deepEqual(dailyUsageBuckets(points),[{day:'2026-10-13',usedPercentagePoints:10,resets:1}]);
});
