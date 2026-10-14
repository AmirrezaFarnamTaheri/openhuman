import {test} from 'node:test';
import assert from 'node:assert/strict';
import {buildCatalog} from '../catalog.mjs';
import {normalizePreset} from '../presets.mjs';
import {FleetController} from '../routing.mjs';
const minimal=(url)=>({id:'ok',name:'OK',provider:'vendor',baseUrl:url,revision:1,models:['model']});
test('model identifiers must not contain HTML injection metacharacters',()=>{
  assert.throws(()=>buildCatalog([], [{provider:'vendor',id:'<img>"'}]));
  assert.throws(()=>buildCatalog([{provider:'vendor',id:'m&x',source:'peer'}],[]));
});
test('remote preset URLs must reject encoded newlines and control characters',()=>{
  assert.throws(()=>normalizePreset(minimal('https://example.com/v1%0aX')));
  assert.throws(()=>normalizePreset(minimal('https://example.com/v1%0d%0aInjected')));
});
test('gateway fleet does not echo an arbitrary account access token through status snapshots',()=>{
  const fleet=new FleetController([{id:'a',provider:'p',models:['m'],accessToken:'super-secret',health:'healthy',remainingPercent:75,quotaFetchedAt:100}],{now:()=>101});
  const view=fleet.snapshot();assert.equal(JSON.stringify(view).includes('super-secret'),false);
});
test('provider declarations with malformed live pricing do not turn into verified cost',()=>{
  const [row]=buildCatalog([], [{provider:'p',id:'m',pricing:{known:true,inputPer1M:-1,outputPer1M:0}}]);
  assert.equal(row.pricing.known,false);
});
