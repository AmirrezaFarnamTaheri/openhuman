import {test} from 'node:test';
import assert from 'node:assert/strict';
import {normalizePreset,mergePresetSync,applyPresetBatch,resolvePresetGroup} from '../presets.mjs';
const preset=(id,revision=1,other={})=>({id,name:id,provider:'google',baseUrl:'https://generativelanguage.googleapis.com/v1beta',models:['m'],revision,...other});
test('preset normalization strips no secrets silently but refuses credential-bearing structures',()=>{
  assert.throws(()=>normalizePreset(preset('a',1,{apiKey:'SECRET'})));
  assert.throws(()=>normalizePreset(preset('a',1,{token:'SECRET'})));
  assert.equal(JSON.stringify(normalizePreset(preset('a'))).includes('SECRET'),false);
});
test('remote HTTP and URL credentials are rejected; local HTTP requires loopback',()=>{
  assert.throws(()=>normalizePreset(preset('a',1,{baseUrl:'http://example.com/v1'})));
  assert.throws(()=>normalizePreset(preset('a',1,{baseUrl:'https://user:pw@example.com/v1'})));
  assert.equal(normalizePreset(preset('a',1,{baseUrl:'http://127.0.0.1:8045/v1'})).baseUrl,'http://127.0.0.1:8045/v1');
  assert.throws(()=>normalizePreset(preset('a',1,{baseUrl:'http://127.0.0.2:8080'})));
});
test('bulk changes are all-or-nothing and require matching revisions',()=>{
  const before=[preset('a')];
  assert.throws(()=>applyPresetBatch(before,[{id:'a',expectedRevision:1,changes:{name:'New'}},{id:'missing',changes:{name:'x'}}]));
  assert.equal(before[0].name,'a');
  const after=applyPresetBatch(before,[{id:'a',expectedRevision:1,changes:{name:'New'}}]);
  assert.equal(after[0].name,'New');assert.equal(after[0].revision,2);
});
test('sync selects higher revision and reports equal-revision concurrent edits',()=>{
  const a=preset('a',2), b=preset('a',3,{name:'remote'});
  assert.equal(mergePresetSync([a],[b]).merged[0].name,'remote');
  assert.equal(mergePresetSync([a],[preset('a',2,{name:'conflict'})]).conflicts.length,1);
});
test('tombstones persist through sync so deleted presets cannot be resurrected',()=>{
  const old=preset('a',1),deleted=preset('a',2,{deleted:true});
  assert.equal(mergePresetSync([deleted],[old]).merged[0].deleted,true);
});
test('a group resolves only configured providers and model IDs',()=>{
  const presets=[preset('a'),preset('b',1,{provider:'openai',models:['x'],groupId:'g'}),preset('c',1,{groupId:'g'})];
  assert.deepEqual(resolvePresetGroup(presets,'g').map(x=>x.id),['b','c']);
});
test('control/bidi model labels are rejected',()=>{
  assert.throws(()=>normalizePreset(preset('a',1,{name:'name\u202e'})));
  assert.throws(()=>normalizePreset(preset('a',1,{models:['m\nfoo']})));
});
