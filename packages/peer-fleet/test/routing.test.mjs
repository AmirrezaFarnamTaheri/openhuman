import {test} from 'node:test';
import assert from 'node:assert/strict';
import {classifyProviderFailure,rankDirectAccounts,FleetController} from '../routing.mjs';
const NOW=1_800_000_000;
const account=(id,remaining=75,other={})=>({id,provider:'google',models:['m'],health:'healthy',remainingPercent:remaining,quotaFetchedAt:NOW-20,inFlight:0,...other});
test('403 validation challenges never become automatic permanent account bans',()=>{
  assert.equal(classifyProviderFailure(403,'VALIDATION_REQUIRED verify account').kind,'verification-required');
  assert.equal(classifyProviderFailure(403,'Access denied').kind,'transient-forbidden');
});
test('429 rate limits and authentication errors remain distinct',()=>{
  assert.equal(classifyProviderFailure(429,'too many requests').kind,'rate-limited');
  assert.equal(classifyProviderFailure(401,'expired').kind,'authentication');
});
test('direct fleet excludes unknown, stale, disabled, exhausted and cooled accounts',()=>{
  const rows=[account('a'),account('b',99,{quotaFetchedAt:NOW-8000}),account('c',100,{health:'disabled'}),account('d',50,{cooldownUntil:NOW+99}),account('e',0)];
  assert.deepEqual(rankDirectAccounts(rows,{provider:'google',model:'m',now:NOW}).map(x=>x.id),['a']);
});
test('direct ranking is deterministic, quota-aware and load-aware',()=>{
  const rows=[account('b',90,{inFlight:3}),account('a',70),account('c',90)];
  assert.deepEqual(rankDirectAccounts(rows,{provider:'google',model:'m',now:NOW}).map(x=>x.id),['c','a','b']);
});
test('external gateway owns its own account pool and must not be double-routed',()=>{
  assert.deepEqual(rankDirectAccounts([account('a')],{provider:'google',model:'m',now:NOW,routeOwner:'prodex'}),[]);
});
test('controller implements idempotent leases and releases',()=>{
  const fleet=new FleetController([account('a')],{now:()=>NOW});
  const first=fleet.acquire({provider:'google',model:'m',requestId:'req'});
  assert.equal(first.accountId,'a');
  assert.deepEqual(fleet.acquire({provider:'google',model:'m',requestId:'req'}),first);
  assert.equal(fleet.snapshot().accounts[0].inFlight,1);
  fleet.release('req',{success:true});fleet.release('req',{success:true});
  assert.equal(fleet.snapshot().accounts[0].inFlight,0);
});
test('controller rejects replay of completed IDs (cannot double-charge)',()=>{
  const fleet=new FleetController([account('a')],{now:()=>NOW});
  fleet.acquire({provider:'google',model:'m',requestId:'x'});fleet.release('x',{success:true});
  assert.throws(()=>fleet.acquire({provider:'google',model:'m',requestId:'x'}));
});
test('sticky session affinity is preserved when account remains eligible',()=>{
  const fleet=new FleetController([account('a',50),account('b',95)],{now:()=>NOW});
  const first=fleet.acquire({provider:'google',model:'m',requestId:'1',sessionKey:'s'}); assert.equal(first.accountId,'b');
  fleet.release('1',{success:true});
  fleet.updateAccounts([account('a',99),account('b',30)]);
  assert.equal(fleet.acquire({provider:'google',model:'m',requestId:'2',sessionKey:'s'}).accountId,'b');
});
test('429 cooldown affects only one account, and no credential is serialized',()=>{
  const fleet=new FleetController([account('a',99,{accessToken:'VERY_SECRET'}),account('b',40)],{now:()=>NOW});
  const l=fleet.acquire({provider:'google',model:'m',requestId:'one'});
  assert.equal(l.accountId,'a'); fleet.release('one',{status:429,body:'too many requests'});
  assert.equal(fleet.acquire({provider:'google',model:'m',requestId:'two'}).accountId,'b');
  assert.equal(JSON.stringify(fleet.snapshot()).includes('VERY_SECRET'),false);
});
test('committed streaming failures must not return an auto-retry instruction',()=>{
  const fleet=new FleetController([account('a')],{now:()=>NOW});
  fleet.acquire({provider:'google',model:'m',requestId:'one'});
  const result=fleet.release('one',{status:503,committed:true});
  assert.equal(result.retryable,false);
});
test('delegated gateway acquisition never exposes or selects an account',()=>{
  const fleet=new FleetController([account('a')],{now:()=>NOW});
  assert.deepEqual(fleet.acquire({provider:'google',model:'m',requestId:'a',routeOwner:'prodex'}),{delegated:true});
  assert.equal(fleet.snapshot().accounts[0].inFlight,0);
});
