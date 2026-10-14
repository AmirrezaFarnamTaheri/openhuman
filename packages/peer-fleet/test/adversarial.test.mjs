import {test} from 'node:test';
import assert from 'node:assert/strict';
import {buildCatalog} from '../catalog.mjs';
import {normalizeQuota} from '../quota.mjs';
import {FleetController,rankDirectAccounts} from '../routing.mjs';
import {dailyUsageBuckets} from '../widgets.mjs';
import {normalizePreset} from '../presets.mjs';
const NOW=1_800_000_000;
const acc=(id,provider='p')=>({id,provider,models:['m'],health:'healthy',remainingPercent:80,quotaFetchedAt:NOW-1});
test('unverified declared context windows cannot appear as verified runtime facts',()=>{
  const [row]=buildCatalog([{id:'m',provider:'p',source:'one',contextWindow:120000}],[]);
  assert.equal(row.contextWindow,null);
  assert.equal(row.declaredContextWindow,120000);
});
test('two contradictory declarations from same source remain marked conflicted',()=>{
  const [row]=buildCatalog([{id:'m',provider:'p',source:'one',contextWindow:2000},{id:'m',provider:'p',source:'one',contextWindow:3000}],[]);
  assert.equal(row.declaredContextWindow,null);
  assert.deepEqual(row.conflicts,['contextWindow']);
});
test('quota parser rejects oversized arrays rather than omitting potentially important windows',()=>{
  assert.throws(()=>normalizeQuota({fetchedAt:NOW,windows:Array.from({length:101},()=>({usedPercent:20}))},NOW));
});
test('fleet refuses an existing account ID being reassigned to a different provider',()=>{
  const fleet=new FleetController([acc('a')],{now:()=>NOW});
  fleet.acquire({provider:'p',model:'m',requestId:'r'});
  assert.throws(()=>fleet.updateAccounts([acc('a','q')]));
});
test('direct fleet must fail closed when owner is not a recognized host or gateway',()=>{
  assert.throws(()=>rankDirectAccounts([acc('a')],{provider:'p',model:'m',now:NOW,routeOwner:'typo'}));
});
test('fleet rejects control characters in account model metadata',()=>{
  assert.throws(()=>new FleetController([{...acc('a'),models:['m\nspoof']}],{now:()=>NOW}));
});
test('daily usage must not invent a daily split from an observation pair across midnight',()=>{
  assert.deepEqual(dailyUsageBuckets([{at:'2026-10-13T23:00:00Z',remainingPercent:60},{at:'2026-10-14T05:00:00Z',remainingPercent:40}]),[]);
});
test('presets reject encoded URL credentials and query keys',()=>{
  const a={id:'a',name:'A',provider:'p',revision:1,models:['m'],baseUrl:'https://example.com/v1?key=hidden'};
  assert.throws(()=>normalizePreset(a));
});
