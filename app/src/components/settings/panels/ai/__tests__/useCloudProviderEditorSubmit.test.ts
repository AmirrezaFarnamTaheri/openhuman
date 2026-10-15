import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { EMPTY_SETTINGS } from '../aiPanelTypes';
import { useCloudProviderEditorSubmit } from '../useCloudProviderEditorSubmit';

const api = vi.hoisted(() => ({
  flushCloudProviders: vi.fn(),
  loadAISettings: vi.fn(),
  listProviderModels: vi.fn(),
  setCloudProviderKey: vi.fn(),
  clearCloudProviderKey: vi.fn(),
}));

vi.mock('../../../../../services/api/aiSettingsApi', async importOriginal => ({
  ...(await importOriginal<typeof import('../../../../../services/api/aiSettingsApi')>()),
  flushCloudProviders: api.flushCloudProviders,
  loadAISettings: api.loadAISettings,
  listProviderModels: api.listProviderModels,
  setCloudProviderKey: api.setCloudProviderKey,
  clearCloudProviderKey: api.clearCloudProviderKey,
}));

beforeEach(() => {
  api.flushCloudProviders.mockReset();
  api.loadAISettings.mockReset();
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
    const persist = vi.fn().mockImplementation(async (nextSettings: typeof saved) => {
      calls.push(nextSettings.cloudProviders.length === 0 ? 'detach' : 'attach');
    });
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: existing, draft: saved, saved, persist,
      t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await result.current({ ...existing, endpoint: 'https://new.example/v1' }, 'replacement-key');
    });
    expect(calls).toEqual(['preview', 'detach', 'store', 'attach']);
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(api.listProviderModels).toHaveBeenCalledWith(
      'existing', 'replacement-key',
      expect.objectContaining({ slug: 'existing', endpoint: 'https://new.example/v1' })
    );
    expect(api.setCloudProviderKey).toHaveBeenCalledWith('existing', 'replacement-key');
    expect(persist).toHaveBeenCalledTimes(2);
    expect(persist.mock.calls[0][0].cloudProviders).toEqual([]);
    expect(persist.mock.calls[1][0].cloudProviders).toEqual([
      expect.objectContaining({ endpoint: 'https://new.example/v1' }),
    ]);
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

  it('leaves a retargeted provider detached if a key-store write fails ambiguously', async () => {
    api.listProviderModels.mockResolvedValue([]);
    api.setCloudProviderKey.mockRejectedValue(new Error('keyring locked'));
    const previous = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [previous] };
    // During key replacement the old route has been detached, so the old
    // credential cannot reach the replacement endpoint.
    api.loadAISettings.mockResolvedValue({ ...saved, cloudProviders: [] });
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
    // An AuthService write may have committed even if its RPC rejected.
    // Restoring the old destination could forward the replacement key there.
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(api.loadAISettings).not.toHaveBeenCalled();
    expect(persist).toHaveBeenCalledTimes(1);
    expect(persist.mock.calls[0][0].cloudProviders).toEqual([]);
  });

  it('does not rewrite unchanged core settings after a rejected provider config save', async () => {
    api.listProviderModels.mockResolvedValue([]);
    api.loadAISettings.mockResolvedValue({ ...EMPTY_SETTINGS, cloudProviders: [] });
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
    expect(api.loadAISettings).toHaveBeenCalledTimes(1);
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
  });
});


describe('provider-submit transport boundary', () => {
  it('rejects keyed remote HTTP even when submit is called directly with probe skipped', async () => {
    const persist = vi.fn();
    const done = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: done,
    }));
    await act(async () => {
      await expect(result.current({
        id: '', slug: 'unsafe-key-route', label: 'Unsafe route',
        endpoint: 'http://external.example/v1',
        authStyle: 'bearer', maskedKey: '',
      }, 'candidate-secret', { skipProbe: true })).rejects.toThrow(/HTTPS|loopback/);
    });
    expect(api.listProviderModels).not.toHaveBeenCalled();
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(persist).not.toHaveBeenCalled();
    expect(done).not.toHaveBeenCalled();
  });

  it('rejects an unsafe image override when a caller bypasses the form validator', async () => {
    const persist = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: vi.fn(),
    }));
    for (const override of [
      { imageEndpoint: 'http://external.example/v1' },
      { modelImageEndpoints: { vision: 'http://169.254.169.254/latest/meta-data' } },
    ]) {
      await act(async () => {
        await expect(result.current({
          id: '', slug: 'external-image', label: 'Unsafe image',
          endpoint: 'https://models.example/v1', authStyle: 'bearer',
          maskedKey: '', ...override,
        }, '', { skipProbe: true })).rejects.toThrow(/HTTPS|loopback/);
      });
    }
    expect(persist).not.toHaveBeenCalled();
    expect(api.listProviderModels).not.toHaveBeenCalled();
  });

  it('continues to permit no-key LAN models without putting secrets on cleartext HTTP', async () => {
    const persist = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await result.current({
        id: '', slug: 'no-key-lan-model', label: 'LAN model',
        endpoint: 'http://192.168.1.5:8000/v1',
        authStyle: 'bearer', maskedKey: '',
      }, '', { skipProbe: true });
    });
    expect(persist).toHaveBeenCalledTimes(1);
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
  });
});


describe('provider save concurrency and rollback isolation', () => {

  it('never restores the old endpoint if final attachment fails after key replacement', async () => {
    api.listProviderModels.mockResolvedValue([]);
    const old = {
      id: 'p_old', slug: 'provider', label: 'Provider',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [old] };
    const persist = vi.fn()
      .mockResolvedValueOnce(undefined)
      .mockRejectedValueOnce(new Error('final attachment failed'));
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: old, draft: saved, saved,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current(
        { ...old, endpoint: 'https://new.example/v1' }, 'replacement-key'
      )).rejects.toThrow(/provider was detached/i);
    });
    expect(persist).toHaveBeenCalledTimes(2);
    expect(persist.mock.calls[0][0].cloudProviders).toEqual([]);
    expect(api.setCloudProviderKey).toHaveBeenCalledWith('provider', 'replacement-key');
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
  });


  it('redacts an echoed candidate credential from preview errors', async () => {
    const secret = 'candidate-sensitive-secret';
    api.listProviderModels.mockRejectedValue(new Error('upstream rejected ' + secret));
    const persist = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current({
        id: '', slug: 'provider-example', label: 'Provider',
        endpoint: 'https://api.example/v1', authStyle: 'bearer', maskedKey: '',
      }, secret)).rejects.toThrow(/\[redacted\]/);
    });
    expect(persist).not.toHaveBeenCalled();
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
  });

  it('refuses overlapping submissions and releases the guard after the first saves', async () => {
    let release: (() => void) | undefined;
    const pending = new Promise<void>(resolve => { release = resolve; });
    const persist = vi.fn().mockReturnValueOnce(pending).mockResolvedValue(undefined);
    const done = vi.fn();
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new', draft: EMPTY_SETTINGS, saved: EMPTY_SETTINGS,
      persist, t: key => key, onDone: done,
    }));
    const provider = {
      id: '', slug: 'safe', label: 'Safe', endpoint: 'https://safe.example/v1',
      authStyle: 'bearer' as const, maskedKey: '',
    };
    let first!: Promise<void>;
    await act(async () => {
      first = result.current(provider, '', { skipProbe: true });
      await expect(result.current(provider, '', { skipProbe: true }))
        .rejects.toThrow(/in progress/i);
    });
    expect(persist).toHaveBeenCalledTimes(1);
    release?.();
    await act(async () => { await first; });
    await act(async () => { await result.current(provider, '', { skipProbe: true }); });
    expect(persist).toHaveBeenCalledTimes(2);
    expect(done).toHaveBeenCalledTimes(2);
  });

  it('does not roll back another editor\'s newer provider settings after a key-store failure', async () => {
    api.listProviderModels.mockResolvedValue([]);
    api.setCloudProviderKey.mockRejectedValue(new Error('keyring locked'));
    const existing = {
      id: 'p_existing', slug: 'existing', label: 'Existing',
      endpoint: 'https://old.example/v1', authStyle: 'bearer' as const,
      maskedKey: '••••stored',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [existing] };
    const persist = vi.fn().mockResolvedValue(undefined);
    api.loadAISettings.mockResolvedValue({
      ...saved,
      cloudProviders: [{
        id: 'p_existing', slug: 'existing', label: 'A concurrent edit',
        endpoint: 'https://concurrent.example/v1', auth_style: 'bearer',
      }],
    });
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: existing, draft: saved, saved,
      persist, t: key => key, onDone: vi.fn(),
    }));
    await act(async () => {
      await expect(result.current(
        { ...existing, endpoint: 'https://new.example/v1' }, 'replacement-key'
      )).rejects.toThrow(/keyring locked/);
    });
    expect(api.loadAISettings).not.toHaveBeenCalled();
    expect(api.flushCloudProviders).not.toHaveBeenCalled();
    expect(persist).toHaveBeenCalledTimes(1);
  });
});
