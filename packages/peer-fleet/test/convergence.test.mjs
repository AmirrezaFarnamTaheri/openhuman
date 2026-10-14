import {test} from 'node:test';
import assert from 'node:assert/strict';
import {composeFleetView, recommendModelRoute, reconcileLiveObservations} from '../convergence.mjs';
const NOW=1_800_000_000;
const account=(id,used=50)=>({id,provider:'antigravity',models:['gemini-3-flash'],health:'healthy',quota:{fetchedAt:NOW-20,windows:[{durationSeconds:18000,usedPercent:used}]}});
const preset={id:'p',name:'Google local',provider:'antigravity',models:['gemini-3-flash'],baseUrl:'https://generativelanguage.googleapis.com/v1beta',groupId:'g',revision:1};
test('end-to-end convergence preserves offline catalog uncertainty while rendering quota widgets',()=>{
  const view=composeFleetView({accounts:[account('a',25)],declaredModels:[{id:'gemini-3-flash',provider:'antigravity',source:'antigravity-panel'}],presets:[preset],groupId:'g',now:NOW});
  assert.equal(view.catalog[0].availability,'unverified');
  assert.equal(view.dashboard.accounts[0].remainingPercent,75);
  assert.equal(view.groups.selected.length,1);
  assert.equal(view.routing.owner,'host');
  assert.equal(view.routing.advisory[0].id,'a');
});
test('delegated Prodex does not cause a second credential fleet or in-memory account rotation',()=>{
  const view=composeFleetView({accounts:[account('a')],presets:[preset],groupId:'g',routeOwner:'prodex',now:NOW});
  assert.deepEqual(view.routing.advisory,[]);
  assert.equal(view.routing.owner,'prodex');
  assert.equal(view.routing.delegated,true);
});
test('selected groups cannot claim offline models are live or service-ready',()=>{
  const view=composeFleetView({accounts:[account('a')],presets:[preset],groupId:'g',declaredModels:[{id:'gemini-3-flash',provider:'antigravity',source:'zero-gravity'}],now:NOW});
  assert.equal(view.groups.selected[0].liveModelCount,0);
});
test('reconciling quota observations refuses stale updates but handles independent account additions',()=>{
  const prev=[account('a',90)];
  const next=[account('a',20),account('b',5)];
  next[0].quota.fetchedAt=NOW-50;
  const out=reconcileLiveObservations(prev,next);
  assert.equal(out[0].quota.windows[0].usedPercent,90);
  assert.equal(out[1].id,'b');
});
test('second-order routing recommendation separates telemetry from runtime action',()=>{
  const a=recommendModelRoute({accounts:[account('a',10),account('b',50)],provider:'antigravity',model:'gemini-3-flash',now:NOW});
  assert.equal(a.mode,'advisory');assert.equal(a.accountId,'a');assert.equal('token' in a,false);
});
test('stale quota cannot be recommended for a direct route',()=>{
  const a=account('a');a.quota.fetchedAt=NOW-9000;
  assert.equal(recommendModelRoute({accounts:[a],provider:'antigravity',model:'gemini-3-flash',now:NOW}).accountId,null);
});
