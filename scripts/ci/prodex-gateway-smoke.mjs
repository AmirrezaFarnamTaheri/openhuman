#!/usr/bin/env node
/**
 * Least-privilege, opt-in Prodex Gateway data-plane diagnostics.
 *
 * No admin, quota, identity, secrets, or management APIs are called. The
 * default probe is read-only. A request that could incur model costs requires
 * both --inference and an explicitly named model advertised by /v1/models.
 * Derived from the Prodex Gateway compatibility contract, not its admin SDK.
 */
import { pathToFileURL } from 'node:url';
import { readFile } from 'node:fs/promises';
import { enrichLiveProdexModels, isSafeProdexIdentifier } from './prodex-catalog-overlay.mjs';

const DEFAULT_BASE_URL = 'http://127.0.0.1:4000/v1';
const DEFAULT_MAX_BYTES = 1024 * 1024;
const DEFAULT_TIMEOUT_MS = 10_000;

// Only errors constructed inside this module may be passed through unchanged.
// Network readers and redirects can throw arbitrary, attacker-controlled text.
class ProdexDataPlaneError extends Error {}

export function normalizeProdexBaseUrl(raw) {
  let url;
  try { url = new URL(raw); } catch { throw new Error('Prodex URL must be a valid HTTP(S) URL'); }
  if (!['http:', 'https:'].includes(url.protocol) || !url.hostname ||
      url.username || url.password || url.search || url.hash) {
    throw new Error('Prodex URL requires HTTP(S) with no embedded credentials, query or fragment');
  }
  const hostname = url.hostname.toLowerCase();
  const loopback = hostname === 'localhost' || hostname === '[::1]' ||
    (hostname.startsWith('127.') && /^127(?:\.[0-9]{1,3}){3}$/.test(hostname));
  if (url.protocol === 'http:' && !loopback) {
    throw new Error('Prodex HTTP endpoints must be loopback; remote gateways require HTTPS');
  }
  const path = url.pathname.replace(/\/+$/, '');
  if (path !== '' && !path.endsWith('/v1')) {
    throw new Error('Prodex API URL must end in /v1');
  }
  url.pathname = (path || '/v1') + '/';
  return url;
}

function scopedKey(raw) {
  const key = typeof raw === 'string' ? raw.trim() : '';
  if (!key || key.length > 4096 || /[\s,;]/.test(key)) {
    throw new Error('Provide exactly one nonempty inference-scoped virtual key');
  }
  return key;
}

function positiveLimit(value, name) {
  if (!Number.isSafeInteger(value) || value < 1) {
    throw new Error(`${name} must be a positive integer`);
  }
  return value;
}

async function boundedJson(response, maxBytes) {
  if (!response.ok) {
    throw new Error(`Prodex data-plane request failed (HTTP ${response.status})`);
  }
  const length = response.headers?.get('content-length');
  if (length !== null && length !== undefined && Number(length) > maxBytes) {
    throw new ProdexDataPlaneError('Prodex data-plane response exceeded size limit');
  }
  if (!response.body) throw new ProdexDataPlaneError('Prodex data-plane response was empty');
  const reader = response.body.getReader();
  const chunks = [];
  let bytes = 0;
  try {
    while (true) {
      const {value, done} = await reader.read();
      if (done) break;
      bytes += value.byteLength;
      if (bytes > maxBytes) {
        await reader.cancel().catch(() => {});
        throw new ProdexDataPlaneError('Prodex data-plane response exceeded size limit');
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }
  const data = new Uint8Array(bytes);
  let offset = 0;
  for (const chunk of chunks) {
    data.set(chunk, offset);
    offset += chunk.byteLength;
  }
  try {
    return JSON.parse(new TextDecoder().decode(data));
  } catch {
    throw new ProdexDataPlaneError('Prodex data-plane response was not valid JSON');
  }
}

async function requestJson({baseUrl, key, method = 'GET', path, payload, fetchImpl, timeoutMs, maxBytes}) {
  const root = normalizeProdexBaseUrl(baseUrl);
  const token = scopedKey(key);
  positiveLimit(timeoutMs, 'timeoutMs');
  positiveLimit(maxBytes, 'maxBytes');
  const controller = new AbortController();
  // Abort built-in fetch *and* enforce a deadline when an injected transport,
  // stalled reader or cancellation hook fails to honor AbortSignal.
  let timeout;
  const deadline = new Promise((_, reject) => {
    timeout = setTimeout(() => {
      controller.abort();
      reject(new ProdexDataPlaneError('Prodex data-plane request timed out'));
    }, timeoutMs);
  });
  try {
    let response;
    try {
      response = await Promise.race([fetchImpl(new URL(path, root), {
        method,
        redirect: 'error',
        headers: {
          authorization: `Bearer ${token}`,
          accept: 'application/json',
          ...(payload === undefined ? {} : {'content-type': 'application/json'}),
        },
        ...(payload === undefined ? {} : {body: JSON.stringify(payload)}),
        signal: controller.signal,
      }), deadline]);
    } catch {
      if (controller.signal.aborted) throw new ProdexDataPlaneError('Prodex data-plane request timed out');
      throw new ProdexDataPlaneError('Prodex data-plane transport or redirect was rejected');
    }
    try {
      return await Promise.race([boundedJson(response, maxBytes), deadline]);
    } catch (error) {
      // Network readers can throw attacker-controlled text that embeds
      // Authorization headers, URL credentials or server response fragments.
      // Only our own bounded parser messages may escape to the caller.
      if (error instanceof ProdexDataPlaneError) {
        throw error;
      }
      throw new ProdexDataPlaneError('Prodex data-plane response could not be read');
    }
  } finally {
    clearTimeout(timeout);
  }
}

export async function probeProdexModels({
  baseUrl = DEFAULT_BASE_URL, key, fetchImpl = fetch,
  timeoutMs = DEFAULT_TIMEOUT_MS, maxBytes = DEFAULT_MAX_BYTES,
}) {
  const result = await requestJson({baseUrl, key, path: 'models', fetchImpl, timeoutMs, maxBytes});
  if (!result || !Array.isArray(result.data) || result.data.length > 10_000) {
    throw new ProdexDataPlaneError('Prodex data-plane model catalog has an invalid shape');
  }
  const models = [];
  const entries = [];
  const ids = new Set();
  for (const item of result.data) {
    const id = item?.id;
    if (!isSafeProdexIdentifier(id) || ids.has(id)) {
      throw new ProdexDataPlaneError('Prodex data-plane model catalog has invalid or duplicate ids');
    }
    if (item.provider !== undefined && item.provider !== null &&
        !isSafeProdexIdentifier(item.provider, 128)) {
      throw new Error('Prodex data-plane model catalog has invalid provider names');
    }
    ids.add(id);
    models.push(id);
    // Only explicitly selected safe gateway facts enter the enrichment layer.
    entries.push({id, ...(typeof item.provider === 'string' ? {provider: item.provider} : {}),
      ...(item.capabilities && typeof item.capabilities === 'object' ? {capabilities: item.capabilities} : {}),
      ...(Number.isSafeInteger(item.context_window) ? {context_window: item.context_window} : {})});
  }
  return {models, entries, modelCount: models.length, status: 'ok'};
}

/**
 * A deliberately opt-in, low-cost native or Chat Completions probe.
 * Never attempts retries/fallback: gateway provider affinity and continuation
 * ownership belong to Prodex, not this health-check client.
 */
export async function probeProdexInference({
  baseUrl = DEFAULT_BASE_URL, key, model, availableModels,
  endpoint = 'chat-completions',
  fetchImpl = fetch, timeoutMs = DEFAULT_TIMEOUT_MS, maxBytes = DEFAULT_MAX_BYTES,
}) {
  if (!['responses', 'chat-completions'].includes(endpoint)) {
    throw new Error('Choose an inference endpoint: responses or chat-completions');
  }
  if (!isSafeProdexIdentifier(model) ||
      !Array.isArray(availableModels) || !availableModels.includes(model)) {
    throw new Error('Requested model is not advertised by the gateway');
  }
  const native = endpoint === 'responses';
  const path = native ? 'responses' : 'chat/completions';
  const payload = native
    ? {model, input: [{role: 'user', content: [{type: 'input_text', text: 'Reply READY.'}]}],
       max_output_tokens: 16, stream: false}
    : {model, messages: [{role: 'user', content: 'Reply READY.'}], max_tokens: 16, stream: false};
  const result = await requestJson({baseUrl, key, method: 'POST', path, payload,
    fetchImpl, timeoutMs, maxBytes});
  if (native) {
    if (result?.object !== 'response' || result.status !== 'completed' || !Array.isArray(result.output)) {
      throw new Error('Prodex Responses probe did not complete with valid output');
    }
    const hasText = result.output.some(item =>
      item?.type === 'message' && item.role === 'assistant' && Array.isArray(item.content) &&
      item.content.some(part => part?.type === 'output_text' &&
        typeof part.text === 'string' && part.text.trim().length > 0));
    if (!hasText) throw new Error('Prodex Responses probe returned no assistant text output');
  } else {
    const hasText = Array.isArray(result?.choices) && result.choices.some(choice =>
      choice?.message?.role === 'assistant' && typeof choice.message.content === 'string' &&
      choice.message.content.trim().length > 0);
    if (!hasText) throw new Error('Prodex inference response did not contain assistant text output');
  }
  const totalTokens = result.usage?.total_tokens;
  return {...(native ? {model, endpoint, status: 'ok'} : {model, status: 'ok'}),
    ...(Number.isSafeInteger(totalTokens) && totalTokens >= 0 ? {totalTokens} : {})};
}

export async function runProdexSmoke(argv = [], env = process.env, fetchImpl = fetch) {
  let inference = false;
  let model;
  let endpoint = 'chat-completions';
  let explicitEndpoint = false;
  let includeCatalog = false;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--inference') inference = true;
    else if (argv[i] === '--catalog') includeCatalog = true;
    else if (argv[i] === '--model' && argv[i + 1]) model = argv[++i];
    else if (argv[i] === '--endpoint' && argv[i + 1]) {
      explicitEndpoint = true;
      endpoint = argv[++i];
    } else throw new Error('Usage: node prodex-gateway-smoke.mjs [--catalog] [--inference --model MODEL [--endpoint responses|chat-completions]]');
  }
  if (!inference && explicitEndpoint) throw new Error('--endpoint requires explicit --inference');
  if (inference && !model) throw new Error('--inference requires --model MODEL');
  if (model && !inference) throw new Error('--model requires explicit --inference');
  if (explicitEndpoint && !['responses', 'chat-completions'].includes(endpoint)) {
    throw new Error('--endpoint must be responses or chat-completions');
  }
  const baseUrl = env.PRODEX_GATEWAY_URL || DEFAULT_BASE_URL;
  const key = env.PRODEX_GATEWAY_CLIENT_KEY;
  const catalog = await probeProdexModels({baseUrl, key, fetchImpl});
  const report = {
    mode: inference ? 'inference' : 'models-only',
    gateway: normalizeProdexBaseUrl(baseUrl).origin,
    models: catalog.models,
    modelCount: catalog.modelCount,
    status: 'ok',
  };
  if (includeCatalog) {
    // Lazy-load untrusted offline hints only when explicitly requested. The
    // gateway's current /models listing alone determines availability.
    const text = await readFile(new URL('./prodex-offline-models.json', import.meta.url), 'utf8');
    let snapshot;
    try { snapshot = JSON.parse(text); }
    catch { throw new Error('Prodex offline catalog snapshot is not valid JSON'); }
    if (snapshot?.schema_version !== 1 || !Array.isArray(snapshot.models) ||
        snapshot.model_count !== snapshot.models.length) {
      throw new Error('Prodex offline catalog snapshot failed validation');
    }
    report.catalog = enrichLiveProdexModels(catalog.entries, snapshot.models);
  }
  if (inference) {
    report.inference = await probeProdexInference({baseUrl, key, model, endpoint,
      availableModels: catalog.models, fetchImpl});
  }
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runProdexSmoke(process.argv.slice(2))
    .then(report => { process.stdout.write(JSON.stringify(report, null, 2) + '\n'); })
    .catch(error => { process.stderr.write(`[prodex-smoke] ${error.message}\n`); process.exitCode = 1; });
}
