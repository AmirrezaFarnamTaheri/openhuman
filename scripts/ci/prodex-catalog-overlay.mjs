/**
 * Untrusted, offline Prodex catalog metadata is *not* an availability or
 * pricing promise. Expose only models actually advertised by a live gateway.
 * No credential, admin API, filesystem or network access in this module.
 */
const MAX_MODELS = 10_000;
const BAD_CONTROL = /[\p{Cc}\p{Cf}\p{Zl}\p{Zp}]/u;

export function isSafeProdexIdentifier(value, maxLength = 256) {
  return typeof value === 'string' && value.length > 0 && value.length <= maxLength &&
    !BAD_CONTROL.test(value);
}

function verifyLive(live) {
  if (!Array.isArray(live) || live.length > MAX_MODELS) {
    throw new TypeError('invalid live model catalog');
  }
  const ids = new Set();
  for (const model of live) {
    const id = model?.id;
    if (!isSafeProdexIdentifier(id) || ids.has(id) ||
        (model.provider !== undefined && !isSafeProdexIdentifier(model.provider, 128))) {
      throw new TypeError('invalid or duplicate live model id/provider');
    }
    ids.add(id);
  }
}

const compare = (a, b) => a.id < b.id ? -1 : a.id > b.id ? 1 : 0;

function explicitCapability(value) {
  return value === true ? 'supported' : value === false ? 'unsupported' : 'unknown';
}

export function enrichLiveProdexModels(live, offline = []) {
  verifyLive(live);
  if (!Array.isArray(offline) || offline.length > MAX_MODELS) {
    throw new TypeError('invalid offline catalog');
  }
  const byId = new Map();
  for (const record of offline) {
    if (!isSafeProdexIdentifier(record?.id) || !isSafeProdexIdentifier(record?.provider, 128)) continue;
    const matches = byId.get(record.id) ?? [];
    matches.push(record);
    byId.set(record.id, matches);
  }
  return live.map(item => {
    const choices = byId.get(item.id) ?? [];
    const qualified = item.provider === undefined
      ? choices
      : choices.filter(x => x.provider === item.provider);
    const match = qualified.length === 1 ? qualified[0] : undefined;
    const catalogStatus = match ? 'matched' :
      qualified.length > 1 || (item.provider === undefined && choices.length > 1)
        ? 'ambiguous' : 'not-in-catalog';
    const label = match?.display_name;
    const contextWindow = Number.isSafeInteger(item.context_window) &&
      item.context_window > 0 ? item.context_window : null;
    // The old catalog cannot certify what the running gateway actually serves.
    // Never present its unknown prices as free or zero-cost services.
    return {
      id: item.id,
      provider: item.provider ?? null,
      displayName: isSafeProdexIdentifier(label, 160) ? label : item.id,
      catalogStatus,
      capabilities: {
        vision: explicitCapability(item.capabilities?.vision),
        tools: explicitCapability(item.capabilities?.tools),
        reasoning: explicitCapability(item.capabilities?.reasoning),
      },
      contextWindow,
      contextWindowVerified: contextWindow !== null,
      pricing: {known: false, inputPer1M: null, outputPer1M: null},
    };
  }).sort(compare);
}
