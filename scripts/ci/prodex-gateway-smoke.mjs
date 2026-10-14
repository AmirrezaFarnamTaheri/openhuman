#!/usr/bin/env node
/**
 * Read-only-by-default Prodex Gateway data-plane diagnostic.
 * This module never invokes management, identity, ledger, quota, or admin APIs.
 * A billable request requires --inference and a catalog-listed --model.
 */
import { pathToFileURL } from 'node:url';

const DEFAULT_BASE_URL = 'http://127.0.0.1:4000/v1';
const DEFAULT_MAX_BYTES = 1024 * 1024;
const DEFAULT_TIMEOUT_MS = 10_000;

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
    throw new Error(name + ' must be a positive integer');
  }
  return value;
}

async function boundedJson(response, maxBytes) {
  if (!response.ok) throw new Error('Prodex data-plane request failed (HTTP ' + response.status + ')');
  const length = response.headers?.get('content-length');
  if (length !== null && length !== undefined && Number(length) > maxBytes) {
    throw new Error('Prodex data-plane response exceeded size limit');
  }
  if (!response.body) throw new Error('Prodex data-plane response was empty');
  const reader = response.body.getReader();
  const chunks = [];
  let bytes = 0;
  try {
    while (true) {
      const { value, done } = await reader.read();
      if (done) break;
      bytes += value.byteLength;
      if (bytes > maxBytes) {
        await reader.cancel().catch(() => {});
        throw new Error('Prodex data-plane response exceeded size limit');
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
    throw new Error('Prodex data-plane response was not valid JSON');
  }
}

async function requestJson({baseUrl, key, method = 'GET', path, payload, fetchImpl, timeoutMs, maxBytes}) {
  const root = normalizeProdexBaseUrl(baseUrl);
  const token = scopedKey(key);
  positiveLimit(timeoutMs, 'timeoutMs');
  positiveLimit(maxBytes, 'maxBytes');
  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);
  timeout.unref?.();
  try {
    let response;
    try {
      response = await fetchImpl(new URL(path, root), {
        method,
        redirect: 'error',
        headers: {
          authorization: 'Bearer ' + token,
          accept: 'application/json',
          ...(payload === undefined ? {} : {'content-type': 'application/json'}),
        },
        ...(payload === undefined ? {} : {body: JSON.stringify(payload)}),
        signal: controller.signal,
      });
    } catch {
      if (controller.signal.aborted) throw new Error('Prodex data-plane request timed out');
      throw new Error('Prodex data-plane transport or redirect was rejected');
    }
    return await boundedJson(response, maxBytes);
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
    throw new Error('Prodex data-plane model catalog has an invalid shape');
  }
  const models = [];
  const ids = new Set();
  for (const item of result.data) {
    const id = item?.id;
    if (typeof id !== 'string' || !id || id.length > 256 || ids.has(id)) {
      throw new Error('Prodex data-plane model catalog has invalid or duplicate ids');
    }
    ids.add(id);
    models.push(id);
  }
  return {models, modelCount: models.length, status: 'ok'};
}

export async function probeProdexInference({
  baseUrl = DEFAULT_BASE_URL, key, model, availableModels,
  fetchImpl = fetch, timeoutMs = DEFAULT_TIMEOUT_MS, maxBytes = DEFAULT_MAX_BYTES,
}) {
  if (typeof model !== 'string' || !availableModels?.includes(model)) {
    throw new Error('Requested model is not advertised by the gateway');
  }
  const result = await requestJson({
    baseUrl, key, method: 'POST', path: 'chat/completions',
    payload: {model, messages: [{role: 'user', content: 'Reply READY.'}], max_tokens: 16, stream: false},
    fetchImpl, timeoutMs, maxBytes,
  });
  if (!Array.isArray(result?.choices) || result.choices.length < 1) {
    throw new Error('Prodex inference response did not contain choices');
  }
  const totalTokens = result.usage?.total_tokens;
  return {model, status: 'ok',
    ...(Number.isSafeInteger(totalTokens) && totalTokens >= 0 ? {totalTokens} : {})};
}

export async function runProdexSmoke(argv = [], env = process.env, fetchImpl = fetch) {
  let inference = false;
  let model;
  for (let i = 0; i < argv.length; i++) {
    if (argv[i] === '--inference') inference = true;
    else if (argv[i] === '--model' && argv[i + 1]) model = argv[++i];
    else throw new Error('Usage: node prodex-gateway-smoke.mjs [--inference --model MODEL]');
  }
  if (inference && !model) throw new Error('--inference requires --model MODEL');
  if (model && !inference) throw new Error('--model requires explicit --inference');
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
  if (inference) report.inference = await probeProdexInference({
    baseUrl, key, model, availableModels: catalog.models, fetchImpl,
  });
  return report;
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  runProdexSmoke(process.argv.slice(2))
    .then(report => { process.stdout.write(JSON.stringify(report, null, 2) + '\n'); })
    .catch(error => { process.stderr.write('[prodex-smoke] ' + error.message + '\n'); process.exitCode = 1; });
}
