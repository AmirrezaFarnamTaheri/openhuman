import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { EMPTY_SETTINGS } from '../aiPanelTypes';
import { useCloudProviderEditorSubmit } from '../useCloudProviderEditorSubmit';

const api = vi.hoisted(() => ({
  flushCloudProviders: vi.fn(),
  listProviderModels: vi.fn(),
  setCloudProviderKey: vi.fn(),
  clearCloudProviderKey: vi.fn(),
}));

vi.mock('../../../../../services/api/aiSettingsApi', async importOriginal => ({
  ...(await importOriginal<typeof import('../../../../../services/api/aiSettingsApi')>()),
  flushCloudProviders: api.flushCloudProviders,
  listProviderModels: api.listProviderModels,
  setCloudProviderKey: api.setCloudProviderKey,
  clearCloudProviderKey: api.clearCloudProviderKey,
}));

beforeEach(() => {
  api.flushCloudProviders.mockReset();
  api.listProviderModels.mockReset();
  api.setCloudProviderKey.mockReset();
  api.clearCloudProviderKey.mockReset();
  api.clearCloudProviderKey.mockResolvedValue(undefined);
  api.setCloudProviderKey.mockResolvedValue(undefined);
});

describe('useCloudProviderEditorSubmit', () => {
  it('persists codec and model transports without an eager flush', async () => {
    const persist = vi.fn().mockResolvedValue(undefined);
    const done = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: done,
    }));
    await act(async () => {
      await result.current({
        id: '', slug: 'qwen-local', label: 'Qwen gateway',
        endpoint: 'http://127.0.0.1:4000/v1', authStyle: 'bearer',
        maskedKey: '', toolCallCodec: 'qnml',
        modelTransports: { 'response-model': 'responses' },
      }, '', { skipProbe: true });
    });
    expect(api.listProviderModels).not.toHaveBeenCalled();
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(persist).toHaveBeenCalledWith(expect.objectContaining({
      cloudProviders: [expect.objectContaining({
        toolCallCodec: 'qnml', modelTransports: { 'response-model': 'responses' },
      })],
    }));
    expect(done).toHaveBeenCalledTimes(1);
  });

  it('probes an edited endpoint without ever exposing the previous key to that endpoint', async () => {
    const calls: string[] = [];
    api.listProviderModels.mockImplementation(async () => { calls.push('preview'); return []; });
    api.setCloudProviderKey.mockImplementation(async () => { calls.push('store'); });
    const existing = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [existing] };
    const persist = vi.fn().mockImplementation(async () => { calls.push('persist'); });
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: existing, draft: saved, saved, persist,
      t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await result.current({ ...existing, endpoint: 'https://new.example/v1' }, 'replacement-key');
    });
    expect(calls).toEqual(['preview', 'persist', 'store']);
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(api.listProviderModels).toHaveBeenCalledWith(
      'existing', 'replacement-key',
      expect.objectContaining({ slug: 'existing', endpoint: 'https://new.example/v1' })
    );
    expect(api.setCloudProviderKey).toHaveBeenCalledWith('existing', 'replacement-key');
  });

  it('keeps both stores untouched if preview verification rejects a new credential', async () => {
    api.listProviderModels.mockRejectedValue(new Error('invalid credential'));
    const existing = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [existing] };
    const persist = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: existing, draft: saved, saved, persist,
      t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current(
        { ...existing, endpoint: 'https://new.example/v1' },
        'bad-key'
      )).rejects.toThrow(/Could not reach/);
    });
    expect(api.listProviderModels).toHaveBeenCalledWith(
      'existing', 'bad-key',
      expect.objectContaining({ endpoint: 'https://new.example/v1' })
    );
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(api.clearCloudProviderKey).not.toHaveBeenCalled();
    expect(persist).not.toHaveBeenCalled();
  });

  it('blocks redirecting a saved key when the endpoint changes, even if verification is skipped', async () => {
    const existing = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [existing] };
    const persist = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: existing, draft: saved, saved,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current(
        { ...existing, endpoint: 'https://new.example/v1' }, '', { skipProbe: true }
      )).rejects.toThrow(/requires a new API key/i);
    });
    expect(api.listProviderModels).not.toHaveBeenCalled();
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(persist).not.toHaveBeenCalled();
  });

  it('allows editing a label without probing or retransmitting a stored key', async () => {
    const existing = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [existing] };
    const persist = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: existing, draft: saved, saved,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await result.current({ ...existing, label: 'Updated label' }, '');
    });
    expect(api.listProviderModels).not.toHaveBeenCalled();
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(persist).toHaveBeenCalledTimes(1);
  });

  it('does not store a new key when the backend rejects a proposed CA certificate', async () => {
    api.listProviderModels.mockRejectedValue(new Error('invalid CA bundle'));
    const persist = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current({
        id: '', slug: 'invalid-ca', label: 'Invalid CA',
        endpoint: 'https://api.example/v1', authStyle: 'bearer',
        maskedKey: '', caCertPem: 'invalid PEM',
      }, 'new-secret')).rejects.toThrow(/invalid CA/);
    });
    expect(api.listProviderModels).toHaveBeenCalledWith(
      'invalid-ca', 'new-secret',
      expect.objectContaining({ ca_cert_pem: 'invalid PEM' })
    );
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(persist).not.toHaveBeenCalled();
  });

  it('compensates settings if secret storage fails after a successful preview', async () => {
    api.listProviderModels.mockResolvedValue([]);
    api.setCloudProviderKey.mockRejectedValue(new Error('keyring locked'));
    const previous = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [previous] };
    const persist = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: previous, draft: saved, saved, persist,
      t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current(
        { ...previous, endpoint: 'https://new.example/v1' }, 'replacement-key'
      )).rejects.toThrow('keyring locked');
    });
    expect(api.flushCloudProviders).toHaveBeenCalledTimes(1);
    expect(api.flushCloudProviders).toHaveBeenCalledWith(expect.arrayContaining([
      expect.objectContaining({ endpoint: 'https://old.example/v1' })
    ]));
    expect(persist).toHaveBeenCalledTimes(2);
    expect(persist).toHaveBeenLastCalledWith(saved);
  });

  it('restores possible partial writes when saving the provider config fails', async () => {
    api.listProviderModels.mockResolvedValue([]);
    const persist = vi.fn().mockRejectedValue(new Error('settings write failed'));
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current({
        id: '', slug: 'new-provider', label: 'New Provider',
        endpoint: 'https://api.example/v1', authStyle: 'bearer', maskedKey: '',
      }, 'new-key')).rejects.toThrow('settings write failed');
    });
    expect(api.flushCloudProviders).toHaveBeenCalledTimes(1);
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
  });
});
