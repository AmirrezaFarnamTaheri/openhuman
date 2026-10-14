import test from 'node:test';
import assert from 'node:assert/strict';
import {normalizeAttemptDiagnostic,aggregateAttemptDiagnostics} from '../telemetry.mjs';

const good={provider:'antigravity',model:'gemini-3-flash',attemptId:'attempt-a',requestedAt:1000,firstFragmentAt:1120,completedAt:1820,outputTokens:210,stream:true,upstreamComplete:true};

test('accepts complete upstream attempts and computes only bounded approximate generation speed',()=>{
 const x=normalizeAttemptDiagnostic(good);
 assert.equal(x.latencyMs,820);
 assert.equal(x.outputGenerationMs,700);
 assert.equal(x.outputTokensPerSecond,300);
 assert(!('apiKey' in x));
});

test('does not mislabel metadata-only, cancelled, failed, or non-stream requests as generation speed',()=>{
 for(const bad of [{...good,outputTokens:99},{...good,stream:false},{...good,upstreamComplete:false},{...good,firstFragmentAt:1810},{...good,failed:true},{...good,firstFragmentAt:950},{...good,attemptId:''}]){
  const x=normalizeAttemptDiagnostic(bad);
  assert.equal(x?.outputTokensPerSecond??null,null);
 }
});

test('rejects invalid timing, malformed identifiers and zero-length success',()=>{
 assert.equal(normalizeAttemptDiagnostic({...good,completedAt:900}),null);
 assert.equal(normalizeAttemptDiagnostic({...good,provider:'???'}),null);
 assert.equal(normalizeAttemptDiagnostic({...good,outputTokens:-1}),null);
});

test('aggregates per-provider model attempts without exposing raw request records',()=>{
 const attempts=[good,{...good,attemptId:'attempt-b',requestedAt:2000,firstFragmentAt:2150,completedAt:3000,outputTokens:500},{...good,attemptId:'attempt-c',failed:true}];
 const out=aggregateAttemptDiagnostics(attempts);
 assert.equal(out[0].attempts,3);
 assert.equal(out[0].successful,2);
 assert.equal(out[0].failed,1);
 assert.equal(out[0].p50LatencyMs,820);
 assert.equal(out[0].p90LatencyMs,1000);
 assert(!JSON.stringify(out).includes('firstFragmentAt'));
});

test('a repeated attempt ID is not double-counted and mixed providers never mix metrics',()=>{
 const s=aggregateAttemptDiagnostics([good,good,{...good,provider:'openai',attemptId:'another'}]);
 assert.equal(s.length,2);
 assert.deepEqual(s.map(x=>x.attempts),[1,1]);
});
