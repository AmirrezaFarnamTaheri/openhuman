import {test} from 'node:test';
import assert from 'node:assert/strict';
import {nextRefreshDue} from '../refresh.mjs';
import {reconcileLiveObservations} from '../convergence.mjs';
const NOW=1_800_000_000;
test('missing last attempt timestamp cannot postpone failed refresh forever',()=>{
  const sample={id:'a',failures:4,active:true,lastFetchedAt:NOW-5000};
  assert.ok(nextRefreshDue(sample,NOW)<=NOW);
  assert.ok(nextRefreshDue(sample,NOW+100)<=NOW+100);
});
test('stale health observations do not overwrite newer health',()=>{
  const prev=[{id:'a',provider:'p',health:'healthy',healthObservedAt:NOW,models:['m'],quota:null}];
  const next=[{id:'a',provider:'p',health:'needs-refresh',healthObservedAt:NOW-100,models:['m'],quota:null}];
  assert.equal(reconcileLiveObservations(prev,next)[0].health,'healthy');
});
test('provider identity mismatch in snapshots fails closed instead of merging two accounts',()=>{
  const a=[{id:'a',provider:'p',models:['m'],health:'healthy'}];
  assert.throws(()=>reconcileLiveObservations(a,[{id:'a',provider:'q',models:['m'],health:'healthy'}]));
});
