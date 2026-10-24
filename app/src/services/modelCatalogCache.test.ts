import { beforeEach, describe, expect, it, vi } from 'vitest';

import { getCoreStateSnapshot, setCoreStateSnapshot } from '../lib/coreState/store';
import type { ModelInfo } from './api/aiSettingsApi';
import {
  cachedModelCatalog,
  invalidateModelCatalogCache,
  loadModelCatalog,
  modelCatalogProviderRevision,
  ModelCatalogSupersededError,
  subscribeModelCatalog,
} from './modelCatalogCache';

function setIdentity(id: string | null) {
  const state = getCoreStateSnapshot();
  setCoreStateSnapshot({
    ...state,
    snapshot: {
      ...state.snapshot,
      auth: { ...state.snapshot.auth, isAuthenticated: id !== null, userId: id },
    },
  });
}

function deferredModels() {
  let resolve!: (models: ModelInfo[]) => void;
  const promise = new Promise<ModelInfo[]>(complete => {
    resolve = complete;
  });
  return { promise, resolve };
}

describe('modelCatalogCache', () => {
  beforeEach(() => {
    invalidateModelCatalogCache();
    setIdentity('A');
  });

  it('shares one in-flight request and reuses its result for the same identity', async () => {
    const pending = deferredModels();
    const load = vi.fn(() => pending.promise);
    const first = loadModelCatalog('private', load);
    const second = loadModelCatalog('private', load);
    expect(first).toBe(second);
    await Promise.resolve();
    expect(load).toHaveBeenCalledOnce();
    const models = [{ id: 'model-A', owned_by: 'private' }];
    pending.resolve(models);
    expect(await first).toEqual(models);
    expect(await loadModelCatalog('private', load)).toEqual(models);
    expect(load).toHaveBeenCalledOnce();
    expect(cachedModelCatalog('private')).toEqual(models);
  });

  it('never returns another identity’s cached catalog', async () => {
    await loadModelCatalog('private', async () => [{ id: 'model-A', owned_by: 'private' }]);
    setIdentity('B');
    expect(cachedModelCatalog('private')).toBeUndefined();
    expect(
      await loadModelCatalog('private', async () => [{ id: 'model-B', owned_by: 'private' }])
    ).toEqual([{ id: 'model-B', owned_by: 'private' }]);
  });

  it('rejects an old identity response that resolves after the account switches', async () => {
    const pending = deferredModels();
    const oldRequest = loadModelCatalog('private', () => pending.promise);
    const rejected = expect(oldRequest).rejects.toBeInstanceOf(ModelCatalogSupersededError);
    await Promise.resolve();
    setIdentity('B');
    const newModels = [{ id: 'model-B', owned_by: 'private' }];
    await loadModelCatalog('private', async () => newModels);
    pending.resolve([{ id: 'model-A', owned_by: 'private' }]);
    await rejected;
    expect(cachedModelCatalog('private')).toEqual(newModels);
  });

  it('rejects a superseded credential response even if it finishes after the replacement read', async () => {
    const pending = deferredModels();
    const stale = loadModelCatalog('private', () => pending.promise);
    const rejected = expect(stale).rejects.toBeInstanceOf(ModelCatalogSupersededError);
    await Promise.resolve();
    invalidateModelCatalogCache('private');
    const newModels = [{ id: 'replacement', owned_by: 'private' }];
    await loadModelCatalog('private', async () => newModels);
    pending.resolve([{ id: 'revoked', owned_by: 'private' }]);
    await rejected;
    expect(cachedModelCatalog('private')).toEqual(newModels);
  });

  it('retries after switching back to an identity whose in-flight read was discarded', async () => {
    const pending = deferredModels();
    const oldRequest = loadModelCatalog('private', () => pending.promise);
    const rejected = expect(oldRequest).rejects.toBeInstanceOf(ModelCatalogSupersededError);
    await Promise.resolve();
    setIdentity('B');
    pending.resolve([{ id: 'obsolete-A', owned_by: 'private' }]);
    await rejected;

    // The committed-identity effect may not observe an intermediate identity
    // during a rapid switch. Its discarded promise must not poison A's cache.
    setIdentity('A');
    const replacement = [{ id: 'fresh-A', owned_by: 'private' }];
    const load = vi.fn(async () => replacement);
    expect(await loadModelCatalog('private', load)).toEqual(replacement);
    expect(load).toHaveBeenCalledOnce();
    expect(cachedModelCatalog('private')).toEqual(replacement);
  });

  it('retries failed reads and leaves other providers cached when one key changes', async () => {
    const failed = loadModelCatalog('private', async () => {
      throw new Error('offline');
    });
    await expect(failed).rejects.toThrow('offline');
    const load = vi.fn(async () => [{ id: 'retry', owned_by: 'private' }]);
    await loadModelCatalog('private', load);
    await loadModelCatalog('other', async () => [{ id: 'other-model', owned_by: 'other' }]);
    invalidateModelCatalogCache('other');
    expect(await loadModelCatalog('private', load)).toEqual([{ id: 'retry', owned_by: 'private' }]);
    expect(load).toHaveBeenCalledOnce();
  });

  it('notifies open consumers and changes only the affected provider revision', () => {
    const listener = vi.fn();
    const unsubscribe = subscribeModelCatalog(listener);
    const before = modelCatalogProviderRevision('private');
    invalidateModelCatalogCache('other');
    expect(modelCatalogProviderRevision('private')).toBe(before);
    expect(modelCatalogProviderRevision('other')).not.toBe(before);
    invalidateModelCatalogCache();
    expect(modelCatalogProviderRevision('private')).not.toBe(before);
    expect(listener).toHaveBeenCalledTimes(2);
    unsubscribe();
  });
});
