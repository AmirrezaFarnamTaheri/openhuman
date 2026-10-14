import {test} from 'node:test';
import assert from 'node:assert/strict';
import {buildCatalog, resolveModelAlias, selectModelPreset} from '../catalog.mjs';

test('catalog does not turn peer declarations into live availability or prices', () => {
  const rows=buildCatalog([{id:'gemini-3-flash',provider:'antigravity',source:'zero-gravity',modalities:['text']}],[]);
  assert.equal(rows[0].availability,'unverified');
  assert.equal(rows[0].pricing.known,false);
  assert.equal(rows[0].capabilities.text,'unknown');
});
test('catalog requires exact provider+model match for live authority', () => {
  const rows=buildCatalog([{id:'same',provider:'antigravity',source:'shield'}], [{id:'same',provider:'openai',capabilities:{tools:true}}]);
  assert.deepEqual(rows.map(x=>x.provider), ['antigravity','openai']);
  assert.equal(rows[0].availability,'unverified');
  assert.equal(rows[1].availability,'live');
});
test('catalog preserves conflicting peer declarations without accepting a winner',()=>{
  const [row]=buildCatalog([{id:'model',provider:'x',source:'peer-a',contextWindow:1024},{id:'model',provider:'x',source:'peer-b',contextWindow:2048}],[]);
  assert.equal(row.contextWindow,null);
  assert.equal(row.conflicts.includes('contextWindow'),true);
  assert.equal(row.sources.length,2);
});
test('catalog rejects duplicate live model keys and unsafe identifiers',()=>{
  assert.throws(()=>buildCatalog([], [{id:'x',provider:'p'}, {id:'x',provider:'p'}]));
  assert.throws(()=>buildCatalog([], [{id:'x\u202e',provider:'p'}]));
});
test('provider-scoped aliases do not cross providers',()=>{
  const aliases=[{provider:'a',alias:'old',target:'new'},{provider:'b',alias:'old',target:'other'}];
  assert.equal(resolveModelAlias('a','old',aliases),'new');
  assert.equal(resolveModelAlias('b','old',aliases),'other');
});
test('aliases fail closed on cycles and shadowing',()=>{
  assert.throws(()=>resolveModelAlias('p','a',[{provider:'p',alias:'a',target:'b'},{provider:'p',alias:'b',target:'a'}]));
  assert.throws(()=>resolveModelAlias('p','a',[{provider:'p',alias:'a',target:'x'},{provider:'p',alias:'a',target:'y'}]));
});
test('image model presets are capability hints until live-verified',()=>{
  const rows=buildCatalog([{id:'image',provider:'antigravity',source:'shield',modalities:['image']}],[]);
  assert.deepEqual(selectModelPreset(rows,'antigravity','image').imageOutput,{state:'unknown'});
});
