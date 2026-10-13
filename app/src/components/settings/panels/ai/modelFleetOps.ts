/** Pure, side-effect-free preset sync and bulk editing.
 * All writes go through the existing OpenHuman encrypted-credential and
 * core-owned config paths; no secrets ever enter this catalog.
 */
import type { CloudProviderCreds, ModelRegistryEntry } from '../../../../utils/tauriCommands/config';
import { BUILTIN_CLOUD_PROVIDERS } from '../builtinCloudProviders';
import { FLEET_MODEL_PRESETS, type FleetModelPreset } from './modelFleetCatalog';

/** Shared normalization with Rust's parse_api_keys; never persist a delimited
 * value as one upstream Authorization header. */
export function parseFleetApiKeys(raw: string): string[] {
  return [...new Set(raw.split(/[,;\r\n]/).map(key => key.trim()).filter(Boolean))];
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
      if (existing && (
        existing.vision !== entry.vision ||
        (existing.context_window ?? 0) !== entry.context_window ||
        (existing.cost_per_1m_input ?? 0) !== entry.cost_per_1m_input ||
        (existing.cost_per_1m_cached_input ?? 0) !== entry.cost_per_1m_cached_input ||
        existing.cost_per_1m_output !== entry.cost_per_1m_output
      )) {
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
  patch: Partial<Pick<ModelRegistryEntry,
    'vision' | 'context_window' | 'cost_per_1m_input' |
    'cost_per_1m_cached_input' | 'cost_per_1m_output'>>
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
  const mutable: Partial<CloudProviderCreds> = {
    ...(patch.image_endpoint !== undefined ? { image_endpoint: patch.image_endpoint } : {}),
    ...(patch.rotation_strategy !== undefined ? { rotation_strategy: patch.rotation_strategy } : {}),
  };
  return providers.map(row => selected.has(row.slug) ? { ...row, ...mutable } : { ...row });
}

/** Reject confusing URL forms and embedded credentials. Loopback HTTP is valid
 * for self-hosted models, as it already is for OpenHuman's local runtime. */
export function validModelFleetEndpoint(value: string): boolean {
  const trimmed = value.trim();
  if (!trimmed) return true; // Blank clears the override.
  try {
    const url = new URL(trimmed);
    return (url.protocol === 'https:' || url.protocol === 'http:') &&
      Boolean(url.hostname) && !url.username && !url.password && !url.search && !url.hash;
  } catch {
    return false;
  }
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
    if (!id || id.length > 200 || /[\u0000-\u001f\u007f]/.test(id)) {
      throw new Error('Image model ids must be nonempty and at most 200 characters.');
    }
    if (!url || !validModelFleetEndpoint(url)) {
      throw new Error('Each model image endpoint must be an http(s) base URL.');
    }
    if (Object.prototype.hasOwnProperty.call(endpoints, id)) {
      throw new Error('Duplicate image model mapping: ' + id);
    }
    endpoints[id] = url;
  }
  return endpoints;
}
