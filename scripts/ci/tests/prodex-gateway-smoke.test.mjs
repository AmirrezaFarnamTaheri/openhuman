import assert from 'node:assert/strict';
import test from 'node:test';
import {
  normalizeProdexBaseUrl,
  probeProdexModels,
  probeProdexInference,
  runProdexSmoke,
} from '../prodex-gateway-smoke.mjs';

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

test('read-only model probe uses scoped token, disabled redirects, bounded bytes and no admin path', async () => {
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
  assert.equal(calls[0].init.headers.authorization, `Bearer ${exampleKey}`);
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

test('rejects multi-key credentials and prevents calls before validation', async () => {
  for (const key of ['', ' ', 'k1,k2', 'k1\nk2', 'k1;k2', 'k1\rk2']) {
    await assert.rejects(
      () => probeProdexModels({baseUrl: 'http://127.0.0.1:4000/v1', key,
        fetchImpl: async () => { throw Error('fetch must not run'); }}),
      /single|key|credential/i,
    );
  }
});

test('inference probe is explicit, no admin token, with only catalog-listed models', async () => {
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

test('CLI default performs read-only probe; requires explicit --inference for billable calls', async () => {
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

test('Responses API probe is explicitly selected and uses the native /v1/responses contract', async () => {
  let requests = 0;
  const fetchImpl = async (url, init) => {
    requests++;
    assert.equal(url.pathname, '/v1/responses');
    assert.equal(init.method, 'POST');
    assert.equal(init.redirect, 'error');
    assert.equal(init.headers.authorization, `Bearer ${exampleKey}`);
    assert.deepEqual(JSON.parse(init.body), {
      model: 'prodex-reasoning',
      input: [{role: 'user', content: [{type: 'input_text', text: 'Reply READY.'}]}],
      max_output_tokens: 16, stream: false,
    });
    return respond(JSON.stringify({object: 'response', status: 'completed',
      output: [{type: 'message', role: 'assistant', content: [{type:'output_text',text:'READY'}]}],
      usage: {input_tokens: 3, output_tokens: 1, total_tokens: 4}}));
  };
  const options = {baseUrl:'https://gateway.example.com/v1',key:exampleKey,
    model:'prodex-reasoning',availableModels:['prodex-reasoning'],endpoint:'responses',fetchImpl};
  const report = await probeProdexInference(options);
  assert.deepEqual(report,{model:'prodex-reasoning', endpoint:'responses',status:'ok',totalTokens:4});
  assert.equal(requests,1);
});

test('native Responses mode rejects malformed/deferred responses without leaking returned bodies', async () => {
  for (const body of [
    {object:'response', status:'in_progress', id:'resp_pending'},
    {object:'response', status:'completed', output: 'untrusted'},
    {object:'response', status:'failed', output:[],error:{message:'SECRET_BODY_NOT_FOR_LOGS'}},
    {object:'wrong',status:'completed',output:[]},
  ]) {
    await assert.rejects(() => probeProdexInference({baseUrl:'https://gateway.example.com/v1',
      key:exampleKey, model:'safe', availableModels:['safe'],endpoint:'responses',
      fetchImpl:async()=>respond(JSON.stringify(body))}),
    error => !String(error).includes('SECRET_BODY_NOT_FOR_LOGS') && !String(error).includes(exampleKey));
  }
});

test('Responses mode remains billable opt-in and never silently falls back to chat completions', async () => {
  const paths = [];
  const env={PRODEX_GATEWAY_CLIENT_KEY:exampleKey,PRODEX_GATEWAY_URL:'http://127.0.0.1:4000/v1'};
  const fetchImpl = async (url) => {
    paths.push(url.pathname);
    if(url.pathname.endsWith('/models')) return respond(validModels);
    assert.equal(url.pathname,'/v1/responses');
    return respond(JSON.stringify({object:'response',status:'completed',output:[{type:'message',role:'assistant',content:[{type:'output_text',text:'READY'}]}]}));
  };
  const result = await runProdexSmoke(['--inference','--model','prodex-fast','--endpoint','responses'],env,fetchImpl);
  assert.equal(result.inference.endpoint,'responses');
  assert.deepEqual(paths,['/v1/models','/v1/responses']);
  paths.length = 0;
  await assert.rejects(() => runProdexSmoke(['--endpoint','responses'],env,fetchImpl),/inference/);
  assert.deepEqual(paths,[]);
  await assert.rejects(() => runProdexSmoke(['--inference','--model','prodex-fast','--endpoint','unknown'],env,fetchImpl),/endpoint/);
  assert.deepEqual(paths,[]);
});

test('model identifier containing terminal control characters cannot be treated as a safe catalog row', async () => {
  await assert.rejects(()=>probeProdexModels({baseUrl:'https://gateway.example.com/v1',key:exampleKey,
    fetchImpl:async()=>respond(JSON.stringify({data:[{id:'model\u001b[31mred'}]}))}),/invalid/);
});

test('response body read errors cannot echo secret transport headers or server content', async () => {
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    headers: {get: () => null},
    body: {getReader: () => ({
      read: async () => {throw new Error(`read failed: Authorization Bearer ${exampleKey}`);},
      releaseLock: () => {},
      cancel: async () => {},
    })},
  });
  await assert.rejects(() => probeProdexModels({
    baseUrl: 'https://gateway.example.com/v1',key:exampleKey,fetchImpl,
  }), error => /response could not be read/.test(String(error)) && !String(error).includes(exampleKey));
});

test('--catalog is a read-only, live-filtered and provenance-aware model inventory', async () => {
  let count=0;
  const env={PRODEX_GATEWAY_CLIENT_KEY:exampleKey,PRODEX_GATEWAY_URL:'http://127.0.0.1:4000/v1'};
  const fetchImpl = async url => {
    count++;
    assert.equal(url.pathname,'/v1/models');
    return respond(JSON.stringify({object:'list',data:[
      {id:'gpt-5.4', provider:'openai', capabilities:{tools:true}},
      {id:'not-in-static-catalog'},
    ]}));
  };
  const result=await runProdexSmoke(['--catalog'],env,fetchImpl);
  assert.equal(count,1);
  assert.equal(result.mode,'models-only');
  assert.equal(result.catalog.length,2);
  assert.equal(result.catalog.find(x=>x.id==='gpt-5.4').catalogStatus,'matched');
  assert.equal(result.catalog.find(x=>x.id==='gpt-5.4').capabilities.tools,'supported');
  assert.equal(result.catalog.find(x=>x.id==='not-in-static-catalog').catalogStatus,'not-in-catalog');
  assert.ok(result.catalog.every(x=>x.pricing.known===false));
  assert.ok(!JSON.stringify(result).includes(exampleKey));
});

test('rejects bidi overrides in advertised model identifiers', async () => {
  await assert.rejects(() => probeProdexModels({baseUrl:'https://gateway.example.com/v1',
    key:exampleKey,fetchImpl:async()=>respond(JSON.stringify({data:[{id:'safe\u202Eevil'}]}))}),/invalid/);
});

test('never trusts network-reader errors merely because their message uses an internal prefix', async () => {
  const fetchImpl = async () => ({
    ok: true,
    headers: {get: () => null},
    body: {getReader: () => ({
      read: async () => { throw new Error(`Prodex data-plane Authorization: Bearer ${exampleKey}`); },
      releaseLock: () => {},
    })},
  });
  await assert.rejects(() => probeProdexModels({
    baseUrl: 'https://gateway.example.com/v1', key: exampleKey, fetchImpl,
  }), error => /response could not be read/.test(String(error)) && !String(error).includes(exampleKey));
});

test('requires real textual assistant output before calling any inference probe successful', async () => {
  const base = {baseUrl: 'https://gateway.example.com/v1', key: exampleKey,
    model: 'advertised', availableModels: ['advertised']};
  for (const body of [
    {choices: [{}]},
    {choices: [{message: {role: 'assistant', content: ''}}]},
    {choices: [{message: {role: 'tool', content: 'READY'}}]},
  ]) {
    await assert.rejects(() => probeProdexInference({...base,
      fetchImpl: async () => respond(JSON.stringify(body))}), /output|choices|assistant/i);
  }
  for (const body of [
    {object: 'response', status: 'completed', output: []},
    {object: 'response', status: 'completed', output: [{type: 'reasoning', summary: []}]},
    {object: 'response', status: 'completed', output: [{type: 'message', role: 'assistant', content: []}]},
  ]) {
    await assert.rejects(() => probeProdexInference({...base, endpoint: 'responses',
      fetchImpl: async () => respond(JSON.stringify(body))}), /output|text/i);
  }
});

test('rejects model/provider terminal line separators consistently in both modes', async () => {
  for (const entry of [{id: 'foo\u2028bar'}, {id: 'safe', provider: 'rogue\u2029name'},
    {id: 'safe', provider: 'bad\r\nHeader: yes'}]) {
    await assert.rejects(() => probeProdexModels({baseUrl: 'https://gateway.example.com/v1',
      key: exampleKey, fetchImpl: async () => respond(JSON.stringify({data: [entry]}))}), /invalid/i);
  }
});

test('timeout also bounds fetches and readers that do not honor AbortSignal', async () => {
  const base = {baseUrl: 'https://gateway.example.com/v1', key: exampleKey, timeoutMs: 30};
  await assert.rejects(() => probeProdexModels({...base,
    fetchImpl: async () => new Promise(() => {})}), /timed out/);
  await assert.rejects(() => probeProdexModels({...base,
    fetchImpl: async () => ({ok: true, status: 200, headers: {get: () => null},
      body: {getReader: () => ({read: async () => new Promise(() => {}), releaseLock: () => {}})}}),
  }), /timed out/);
});
