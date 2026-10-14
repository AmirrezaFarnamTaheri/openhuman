import assert from 'node:assert/strict';
import test from 'node:test';
import {enrichLiveProdexModels} from '../prodex-catalog-overlay.mjs';

const local = [
 {provider:'openai', id:'shared',display_name:'OpenAI Shared',feature_flags:{vision:false,tools:true},pricing_known:false,context_window_tokens:128000},
 {provider:'copilot',id:'shared',display_name:'Copilot Shared',feature_flags:{vision:true,tools:true},pricing_known:false,context_window_tokens:512000},
 {provider:'gemini',id:'unique',display_name:'Gemini Unique',feature_flags:{vision:true,tools:true},pricing_known:false,context_window_tokens:999999},
 {provider:'gemini',id:'only-offline',display_name:'Offline Only',feature_flags:{vision:true},pricing_known:false},
];

test('never treats catalog-only models as available or picks arbitrary provider on collisions', () => {
 const entries = enrichLiveProdexModels([{id:'shared'},{id:'unique'},{id:'not-in-static'}],local);
 assert.deepEqual(entries.map(x=>x.id),['not-in-static','shared','unique']);
 const shared=entries.find(x=>x.id==='shared');
 assert.equal(shared.catalogStatus,'ambiguous');
 assert.equal(shared.displayName,'shared');
 assert.equal(shared.capabilities.vision,'unknown');
 assert.ok(!entries.some(x=>x.id==='only-offline'));
 const novel=entries.find(x=>x.id==='not-in-static');
 assert.equal(novel.catalogStatus,'not-in-catalog');
 assert.equal(novel.pricing.known,false);
});

test('provider hint disambiguates offline label but not its unverified capabilities or price',()=>{
 const [entry]=enrichLiveProdexModels([{id:'shared',provider:'copilot'}],local);
 assert.equal(entry.displayName,'Copilot Shared');
 assert.equal(entry.catalogStatus,'matched');
 assert.equal(entry.provider,'copilot');
 assert.equal(entry.capabilities.vision,'unknown');
 assert.equal(entry.capabilities.tools,'unknown');
 assert.equal(entry.contextWindowVerified,false);
 assert.equal(entry.pricing.known,false);
 assert.equal(entry.pricing.inputPer1M,null);
});

test('capabilities require agreement with explicitly announced live facts',()=>{
 const [entry]=enrichLiveProdexModels([{id:'unique',provider:'gemini',capabilities:{vision:true,tools:false},context_window:32000}],local);
 assert.equal(entry.capabilities.vision,'supported');
 assert.equal(entry.capabilities.tools,'unsupported');
 assert.equal(entry.contextWindow,32000);
 assert.equal(entry.contextWindowVerified,true);
});

test('invalid live IDs, duplicates, unexpected providers, and control chars are rejected',()=>{
 for(const live of [[{id:''}],[{id:'x\u001b[31m'}],[{id:'x'},{id:'x'}],[{id:'x',provider:'evil\r\n'}]]) {
  assert.throws(()=>enrichLiveProdexModels(live,local),/invalid|duplicate/);
 }
});

test('does not mutate source records and uses stable input-order-independent sorting',()=>{
 const snapshot=JSON.stringify(local);
 const result=enrichLiveProdexModels([{id:'unique',provider:'gemini'},{id:'shared',provider:'copilot'}],local);
 assert.deepEqual(result.map(x=>x.id),['shared','unique']);
 assert.equal(JSON.stringify(local),snapshot);
 assert.equal(result[0].catalogStatus,'matched');
});

test('blocks Unicode line separators and control characters in live provider names and IDs', () => {
  for (const model of [{id: 'm\u2028break'}, {id: 'm', provider: 'bad\u2029provider'}]) {
    assert.throws(() => enrichLiveProdexModels([model], local), /invalid/);
  }
});

test('offline provider hints never masquerade as verified live provider metadata', () => {
  const [row] = enrichLiveProdexModels([{id: 'unique'}], local);
  assert.equal(row.catalogStatus, 'matched');
  assert.equal(row.provider, null);
  assert.equal(row.displayName, 'Gemini Unique');
});
