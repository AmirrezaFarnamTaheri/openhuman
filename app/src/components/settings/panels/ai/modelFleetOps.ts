/** Pure, side-effect-free preset sync and bulk editing.
 * All writes go through the existing OpenHuman encrypted-credential and
 * core-owned config paths; no secrets ever enter this catalog.
 */
import type {
  CloudProviderCreds,
  ModelRegistryEntry,
  ModelTransport,
} from '../../../../utils/tauriCommands/config';
import { BUILTIN_CLOUD_PROVIDERS } from '../builtinCloudProviders';
import { FLEET_MODEL_PRESETS, type FleetModelPreset } from './modelFleetCatalog';

/** Shared normalization with Rust's parse_api_keys; never persist a delimited
 * value as one upstream Authorization header. */
export function parseFleetApiKeys(raw: string): string[] {
  return [
    ...new Set(
      raw
        .split(/[,;\r\n]/)
        .map(key => key.trim())
        .filter(Boolean)
    ),
  ];
}

/** Insert a pasted credential group without accidentally deleting the
 * credential already in an occupied row. A collapsed selection inserts *rows*
 * beside the existing key; a deliberate selected replacement replaces the
 * selection. Never truncate a batch silently at the input-row cap.
 */
export function insertFleetKeysAtSelection(
  rows: readonly string[],
  rowIndex: number,
  pastedText: string,
  selectionStart: number,
  selectionEnd: number,
  limit = 128
): string[] {
  if (!Number.isInteger(rowIndex) || rowIndex < 0 || rowIndex >= rows.length) {
    throw new Error('Invalid API key row');
  }
  const existing = rows[rowIndex] ?? '';
  const start = Math.max(0, Math.min(existing.length, selectionStart));
  const end = Math.max(start, Math.min(existing.length, selectionEnd));
  const pasted = parseFleetApiKeys(pastedText);
  if (pasted.length === 0) return [...rows];

  let replacement: string[];
  if (start === end && existing.trim()) {
    // No explicit replacement: preserve the existing credential as a row.
    replacement = [existing, ...pasted];
  } else if (!existing.trim() || (start === 0 && end === existing.length)) {
    replacement = pasted;
  } else {
    // The user deliberately selected text to replace inside a row.
    replacement = parseFleetApiKeys(existing.slice(0, start) + pastedText + existing.slice(end));
  }
  const next = [...rows.slice(0, rowIndex), ...replacement, ...rows.slice(rowIndex + 1)];
  if (next.length > limit) {
    throw new Error(`A provider group supports at most ${limit} API keys.`);
  }
  return next;
}

export const fleetModelKey = (provider: string, id: string) => provider + '\0' + id;

export function syncModelPresets(
  current: readonly ModelRegistryEntry[],
  presets: readonly FleetModelPreset[] = FLEET_MODEL_PRESETS
): { models: ModelRegistryEntry[]; added: number; updated: number } {
  const next = current.map(entry => ({ ...entry }));
  const byKey = new Map(next.map((row, index) => [fleetModelKey(row.provider, row.id), index]));
  let added = 0;
  let updated = 0;
  for (const preset of presets) {
    const key = fleetModelKey(preset.provider, preset.id);
    const entry: ModelRegistryEntry = {
      id: preset.id,
      provider: preset.provider,
      context_window: preset.context_window,
      vision: preset.vision,
      cost_per_1m_input: preset.cost_per_1m_input,
      cost_per_1m_cached_input: preset.cost_per_1m_cached_input,
      cost_per_1m_output: preset.cost_per_1m_output,
    };
    const idx = byKey.get(key);
    if (idx === undefined) {
      byKey.set(key, next.length);
      next.push(entry);
      added++;
    } else {
      const existing = next[idx];
      if (
        existing &&
        (existing.vision !== entry.vision ||
          (existing.context_window ?? 0) !== entry.context_window ||
          (existing.cost_per_1m_input ?? 0) !== entry.cost_per_1m_input ||
          (existing.cost_per_1m_cached_input ?? 0) !== entry.cost_per_1m_cached_input ||
          existing.cost_per_1m_output !== entry.cost_per_1m_output)
      ) {
        next[idx] = { ...existing, ...entry };
        updated++;
      }
    }
  }
  return { models: next, added, updated };
}

export function applyModelBulkPatch(
  models: readonly ModelRegistryEntry[],
  selected: ReadonlySet<string>,
  patch: Partial<
    Pick<
      ModelRegistryEntry,
      | 'vision'
      | 'context_window'
      | 'cost_per_1m_input'
      | 'cost_per_1m_cached_input'
      | 'cost_per_1m_output'
    >
  >
): ModelRegistryEntry[] {
  return models.map(model =>
    selected.has(fleetModelKey(model.provider, model.id)) ? { ...model, ...patch } : { ...model }
  );
}

type ManagedFields = Pick<CloudProviderCreds, 'slug' | 'endpoint' | 'auth_style'>;
type SyncedProvider<T extends ManagedFields> = Omit<T, 'endpoint' | 'auth_style'> & ManagedFields;
export function syncConfiguredProviderPresets<T extends ManagedFields>(
  providers: readonly T[]
): { providers: Array<SyncedProvider<T>>; updated: number } {
  const catalog = new Map(BUILTIN_CLOUD_PROVIDERS.map(p => [p.slug, p]));
  let updated = 0;
  const next: Array<SyncedProvider<T>> = providers.map(row => {
    const preset = catalog.get(row.slug);
    if (!preset) return { ...row };
    if (row.endpoint === preset.endpoint && row.auth_style === preset.authStyle) return { ...row };
    updated++;
    // Existing labels, API-key reference, image mapping, and rotation preferences
    // are user-owned and never overwritten by a catalog synchronization.
    return { ...row, endpoint: preset.endpoint, auth_style: preset.authStyle } as SyncedProvider<T>;
  });
  return { providers: next, updated };
}

export function applyProviderBulkPatch<T extends { slug: string }>(
  providers: readonly T[],
  selected: ReadonlySet<string>,
  patch: Partial<CloudProviderCreds>
): T[] {
  // Model-specific and group-wide image requests reuse real bearer/API keys.
  // Never allow a bulk caller to install an off-loopback plaintext endpoint.
  if (patch.image_endpoint != null && !validModelFleetImageEndpoint(patch.image_endpoint)) {
    throw new Error('Image endpoints require HTTPS or loopback HTTP.');
  }
  const mutable: Partial<CloudProviderCreds> = {
    ...(patch.image_endpoint !== undefined ? { image_endpoint: patch.image_endpoint } : {}),
    ...(patch.rotation_strategy !== undefined
      ? { rotation_strategy: patch.rotation_strategy }
      : {}),
  };
  return providers.map(row => (selected.has(row.slug) ? { ...row, ...mutable } : { ...row }));
}

/** Reject confusing URL forms and embedded credentials. Loopback HTTP is valid
 * for self-hosted models, as it already is for OpenHuman's local runtime. */
export function validModelFleetEndpoint(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return true; // Blank clears the override.
  try {
    const url = new URL(trimmed);
    return (
      (url.protocol === 'https:' || url.protocol === 'http:') &&
      Boolean(url.hostname) &&
      !url.username &&
      !url.password &&
      !url.search &&
      !url.hash
    );
  } catch {
    return false;
  }
}

/** A model-specific image route receives the same provider key as text
 * inference. Never send it over remote HTTP. Match Rust's loopback exemption,
 * but do not change legacy primary-provider endpoint validation here.
 */
export function validModelFleetImageEndpoint(value: string): boolean {
  if (!validModelFleetEndpoint(value)) return false;
  const trimmed = value.trim();
  if (!trimmed) return true;
  const url = new URL(trimmed);
  if (url.protocol === 'https:') return true;
  if (url.protocol !== 'http:') return false;
  const host = url.hostname.toLowerCase();
  return (
    host === 'localhost' ||
    host === '[::1]' ||
    /^127(?:\.\d{1,3}){3}$/.test(host)
  );
}

/** A blank image-URL input means 'do not edit' unless Clear was clicked;
 * applying a strategy must not silently clear the existing endpoint.
 */
export function updateSelectedGroupSettings<
  T extends {
    slug: string;
    imageEndpoint?: string;
    rotationStrategy?: 'round_robin' | 'least_busy' | 'priority_weighted';
  },
>(
  groups: readonly T[],
  selected: ReadonlySet<string>,
  patch: {
    imageEndpoint?: string;
    rotationStrategy?: 'round_robin' | 'least_busy' | 'priority_weighted';
  }
): T[] {
  // Enforce the destination boundary for ALL callers, including preset or
  // future bulk-edit surfaces that bypass the current panel's form check.
  if (patch.imageEndpoint !== undefined && !validModelFleetImageEndpoint(patch.imageEndpoint)) {
    throw new Error('Image endpoints require HTTPS or loopback HTTP.');
  }
  // The settings model uses the full enum. A caller may have narrower inferred
  // literals, but updating rotation always retains the same runtime structure.
  return groups.map(group =>
    selected.has(group.slug) ? { ...group, ...patch } : { ...group }
  ) as T[];
}

/** Explicit model-id-to-protocol mapping. Unknown models remain Auto.
 * Keep a null-prototype object so user-provided ids cannot mutate its prototype.
 */
export function parseModelTransportOverrides(input: string): Record<string, ModelTransport> {
  const routes: Record<string, ModelTransport> = Object.create(null) as Record<
    string,
    ModelTransport
  >;
  let count = 0;
  for (const line of input.split(/\r?\n/)) {
    if (!line.trim()) continue;
    const separator = line.indexOf('=');
    if (separator < 1) throw new Error('Each model protocol override must be model-id = protocol.');
    const id = line.slice(0, separator).trim();
    const protocol = line.slice(separator + 1).trim();
    if (!id || id.length > 200 || /\p{Cc}/u.test(id)) {
      throw new Error('Model ids must be 1-200 printable characters.');
    }
    if (!['auto', 'chat_completions', 'responses'].includes(protocol)) {
      throw new Error('Protocol must be auto, chat_completions, or responses.');
    }
    if (Object.prototype.hasOwnProperty.call(routes, id)) {
      throw new Error('Duplicate model protocol override: ' + id);
    }
    if (++count > 512) throw new Error('At most 512 model protocol overrides are supported.');
    routes[id] = protocol as ModelTransport;
  }
  return routes;
}

/** Explicit model-id-to-endpoint group keys, one `model-id = https://...` per line. */
export function parseModelImageEndpoints(input: string): Record<string, string> {
  const endpoints: Record<string, string> = Object.create(null) as Record<string, string>;
  for (const line of input.split(/\r?\n/)) {
    if (!line.trim()) continue;
    const sep = line.indexOf('=');
    if (sep < 1) throw new Error('Each image model mapping must be model-id = base URL.');
    const id = line.slice(0, sep).trim();
    const url = line.slice(sep + 1).trim();
    if (!id || id.length > 200 || /\p{Cc}/u.test(id)) {
      throw new Error('Image model ids must be nonempty and at most 200 characters.');
    }
    if (!url || !validModelFleetImageEndpoint(url)) {
      throw new Error('Image endpoints require HTTPS or loopback HTTP, without URL credentials.');
    }
    if (Object.prototype.hasOwnProperty.call(endpoints, id)) {
      throw new Error('Duplicate image model mapping: ' + id);
    }
    endpoints[id] = url;
  }
  return endpoints;
}
