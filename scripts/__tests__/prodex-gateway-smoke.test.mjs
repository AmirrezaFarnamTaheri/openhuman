import assert from 'node:assert/strict';
import test from 'node:test';
import {
  normalizeProdexBaseUrl,
  probeProdexModels,
  probeProdexInference,
  runProdexSmoke,
} from '../ci/prodex-gateway-smoke.mjs';

const exampleKey = 'virtual-key-test-never-print';
const validModels = JSON.stringify({object: 'list', data: [
  {id: 'prodex-fast', object: 'model'}, {id: 'prodex-reasoning', object: 'model'},
]});

function respond(text = validModels, options = {}) {
  return new Response(text, {
    status: options.status ?? 200,
    headers: {'content-type': 'application/json', ...(options.headers ?? {})},
  });
}

test('accepts HTTPS remote and HTTP loopback, but refuses credential-leaking URLs', () => {
  for (const url of [
    'https://gateway.example.com/v1', 'http://localhost:4000/v1',
    'http://127.0.0.1:4000/v1', 'http://[::1]:4000/v1',
  ]) assert.ok(normalizeProdexBaseUrl(url).pathname.endsWith('/v1/'), url);
  for (const url of [
    'http://gateway.example.com/v1', 'http://127.0.0.1.evil.com:4000/v1',
    'https://api.example.com/v1?token=secret',
    'https://api.example.com/v1#fragment',
    'https://user:pass@gateway.example.com/v1',
    'file:///etc/passwd', 'http://0.0.0.0:4000/v1',
  ]) assert.throws(() => normalizeProdexBaseUrl(url), undefined, url);
});

test('read-only model probe sends scoped token, refuses redirects, limits response and never calls admin', async () => {
  const calls = [];
  const fetchImpl = async (url, init) => {
    calls.push({url: url.toString(), init});
    return respond();
  };
  const result = await probeProdexModels({
    baseUrl: 'http://127.0.0.1:4000/v1', key: exampleKey, fetchImpl,
  });
  assert.deepEqual(result.models, ['prodex-fast', 'prodex-reasoning']);
  assert.equal(calls.length, 1);
  assert.equal(calls[0].url, 'http://127.0.0.1:4000/v1/models');
  assert.equal(calls[0].init.headers.authorization, 'Bearer ' + exampleKey);
  assert.equal(calls[0].init.redirect, 'error');
  assert.equal(calls[0].init.method, 'GET');
  assert.equal(JSON.stringify(result).includes(exampleKey), false);
});

test('rejects oversized, malformed, failed and duplicate model results without echoing bodies', async () => {
  for (const fake of [
    () => respond('SECRET_BODY_NOT_FOR_LOGS', {status: 401}),
    () => respond('malformed{'),
    () => respond(JSON.stringify({data: [{id: 'm'}, {id: 'm'}]})),
    () => respond(JSON.stringify({data: [{id: ''}]})),
    () => respond(validModels, {headers: {'content-length': '999999999'}}),
  ]) {
    await assert.rejects(
      () => probeProdexModels({baseUrl: 'https://gateway.example.com/v1',
        key: exampleKey, maxBytes: 1000, fetchImpl: async () => fake()}),
      error => !String(error).includes(exampleKey) && !String(error).includes('SECRET_BODY_NOT_FOR_LOGS'),
    );
  }
});

test('rejects multi-key credentials before any request is sent', async () => {
  for (const key of ['', ' ', 'k1,k2', 'k1\nk2', 'k1;k2', 'k1\rk2']) {
    await assert.rejects(
      () => probeProdexModels({baseUrl: 'http://127.0.0.1:4000/v1', key,
        fetchImpl: async () => { throw Error('fetch must not run'); }}),
      /single|key|credential/i,
    );
  }
});

test('inference requires a catalog-listed model and never exposes completion content in report', async () => {
  let calls = 0;
  const fetchImpl = async (url, init) => {
    calls++;
    assert.ok(url.toString().endsWith('/v1/chat/completions'));
    assert.equal(init.method, 'POST');
    assert.equal(init.redirect, 'error');
    assert.deepEqual(JSON.parse(init.body).messages, [{role:'user', content:'Reply READY.'}]);
    return respond(JSON.stringify({choices: [{message: {role: 'assistant', content: 'READY'}}], usage: {total_tokens: 4}}));
  };
  await assert.rejects(() => probeProdexInference({
    baseUrl: 'http://127.0.0.1:4000/v1', key: exampleKey,
    model: 'unadvertised', availableModels: ['prodex-fast'], fetchImpl,
  }), /not advertised/);
  assert.equal(calls, 0);
  const result = await probeProdexInference({baseUrl:'http://127.0.0.1:4000/v1',key:exampleKey,
    model:'prodex-fast', availableModels:['prodex-fast'],fetchImpl});
  assert.equal(calls, 1);
  assert.deepEqual(result, {model:'prodex-fast',status:'ok',totalTokens:4});
});

test('CLI defaults to read-only mode and requires explicit billable-inference opt-in', async () => {
  let modelCalls = 0;
  let inferenceCalls = 0;
  const fetchImpl = async (url) => {
    if (url.toString().endsWith('/models')) { modelCalls++; return respond(); }
    inferenceCalls++;
    return respond(JSON.stringify({choices:[{message:{role:'assistant',content:'READY'}}]}));
  };
  const env = {PRODEX_GATEWAY_CLIENT_KEY:exampleKey,PRODEX_GATEWAY_URL:'http://127.0.0.1:4000/v1'};
  const report = await runProdexSmoke([], env, fetchImpl);
  assert.equal(report.mode, 'models-only');
  assert.equal(modelCalls, 1);
  assert.equal(inferenceCalls, 0);
  await assert.rejects(() => runProdexSmoke(['--inference'], env, fetchImpl), /--model/);
  const billed = await runProdexSmoke(['--inference','--model','prodex-fast'], env, fetchImpl);
  assert.equal(billed.mode, 'inference');
  assert.equal(inferenceCalls, 1);
});

// Positive probes must require actual assistant output; metadata-only
// completions cannot be reported as healthy or billed inference.
test('metadata-only completion is not a successful inference probe', async () => {
  await assert.rejects(() => probeProdexInference({
    baseUrl: 'http://127.0.0.1:4000/v1', key: exampleKey,
    model: 'prodex-fast', availableModels: ['prodex-fast'],
    fetchImpl: async () => respond(JSON.stringify({choices: [{message: {role: 'assistant', content: ''}}]})),
  }), /assistant text output/);
});
