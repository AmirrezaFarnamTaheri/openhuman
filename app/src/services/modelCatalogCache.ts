import { type CoreAppSnapshot, getCoreStateSnapshot } from '../lib/coreState/store';
import type { ModelInfo } from './api/aiSettingsApi';

interface CatalogEntry {
  identity: string;
  models?: ModelInfo[];
  request: Promise<ModelInfo[]>;
}

const catalogs = new Map<string, CatalogEntry>();
const providerRevisions = new Map<string, number>();
const listeners = new Set<() => void>();
let revision = 0;
let resetRevision = 0;

/** Credentials and model access belong to the committed core identity. */
export function modelCatalogIdentity(snapshot: CoreAppSnapshot = getCoreStateSnapshot().snapshot) {
  return JSON.stringify([
    snapshot.auth.isAuthenticated,
    snapshot.auth.userId ?? snapshot.currentUser?._id ?? null,
    snapshot.auth.profileId,
  ]);
}

export const modelCatalogRevision = () => revision;
export const modelCatalogProviderRevision = (slug: string | null) =>
  (slug ? providerRevisions.get(slug) : undefined) ?? resetRevision;

export function subscribeModelCatalog(listener: () => void): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

/** Invalidates resolved and in-flight reads, and refreshes any open picker. */
export function invalidateModelCatalogCache(providerSlug?: string): void {
  revision += 1;
  if (providerSlug) {
    catalogs.delete(providerSlug);
    providerRevisions.set(providerSlug, revision);
  } else {
    catalogs.clear();
    providerRevisions.clear();
    resetRevision = revision;
  }
  listeners.forEach(listener => listener());
}

export function cachedModelCatalog(providerSlug: string): ModelInfo[] | undefined {
  const entry = catalogs.get(providerSlug);
  return entry?.identity === modelCatalogIdentity() ? entry.models : undefined;
}

export class ModelCatalogSupersededError extends Error {
  constructor() {
    super('The model catalog changed while it was loading.');
    this.name = 'ModelCatalogSupersededError';
  }
}

/** Coalesce concurrent reads without allowing an obsolete response to repopulate the cache. */
export function loadModelCatalog(
  providerSlug: string,
  load: () => Promise<ModelInfo[]>
): Promise<ModelInfo[]> {
  const identity = modelCatalogIdentity();
  const cached = catalogs.get(providerSlug);
  if (cached?.identity === identity) return cached.request;

  const entry: CatalogEntry = { identity, request: Promise.resolve().then(load) };
  entry.request = entry.request.then(
    models => {
      if (catalogs.get(providerSlug) !== entry || modelCatalogIdentity() !== identity) {
        if (catalogs.get(providerSlug) === entry) catalogs.delete(providerSlug);
        throw new ModelCatalogSupersededError();
      }
      entry.models = models;
      return models;
    },
    error => {
      if (catalogs.get(providerSlug) === entry) catalogs.delete(providerSlug);
      throw error;
    }
  );
  catalogs.set(providerSlug, entry);
  return entry.request;
}
