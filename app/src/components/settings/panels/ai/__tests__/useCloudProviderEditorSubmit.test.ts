import { act, renderHook } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { EMPTY_SETTINGS } from '../aiPanelTypes';
import { useCloudProviderEditorSubmit } from '../useCloudProviderEditorSubmit';

const api = vi.hoisted(() => ({
  flushCloudProviders: vi.fn(),
  listProviderModels: vi.fn(),
  setCloudProviderKey: vi.fn(),
}));

vi.mock('../../../../../services/api/aiSettingsApi', async importOriginal => ({
  ...(await importOriginal<typeof import('../../../../../services/api/aiSettingsApi')>()),
  flushCloudProviders: api.flushCloudProviders,
  listProviderModels: api.listProviderModels,
  setCloudProviderKey: api.setCloudProviderKey,
}));

beforeEach(() => {
  api.flushCloudProviders.mockReset();
  api.listProviderModels.mockReset();
  api.setCloudProviderKey.mockReset();
  api.setCloudProviderKey.mockResolvedValue(undefined);
});

describe('useCloudProviderEditorSubmit', () => {
  it('persists QNML compatibility through the native verified provider editor', async () => {
    api.flushCloudProviders.mockResolvedValue(undefined);
    const persist = vi.fn().mockResolvedValue(undefined);
    const onDone = vi.fn();
    const { result } = renderHook(() =>
      useCloudProviderEditorSubmit({
        editing: 'new',
        draft: EMPTY_SETTINGS,
        saved: EMPTY_SETTINGS,
        persist,
        t: key => key,
        onDone,
      })
    );

    await act(async () => {
      await result.current(
        {
          id: '',
          slug: 'qwen-2api-local',
          label: 'Local Qwen gateway',
          endpoint: 'http://127.0.0.1:4000/v1',
          authStyle: 'bearer',
          toolCallCodec: 'qnml',
          maskedKey: '',
        },
        '',
        { skipProbe: true }
      );
    });

    expect(api.flushCloudProviders).toHaveBeenCalledWith(
      expect.arrayContaining([
        expect.objectContaining({
          slug: 'qwen-2api-local',
          tool_call_codec: 'qnml',
        }),
      ])
    );
    expect(persist).toHaveBeenCalledWith(
      expect.objectContaining({
        cloudProviders: [
          expect.objectContaining({ slug: 'qwen-2api-local', toolCallCodec: 'qnml' }),
        ],
      })
    );
    expect(onDone).toHaveBeenCalledOnce();
  });

  it('retains per-model protocol selection in eager provider flush and saved configuration', async () => {
    api.flushCloudProviders.mockResolvedValue(undefined);
    const persist = vi.fn().mockResolvedValue(undefined);
    const { result } = renderHook(() => useCloudProviderEditorSubmit({
      editing: 'new',
      draft: EMPTY_SETTINGS,
      saved: EMPTY_SETTINGS,
      persist,
      t: key => key,
      onDone: vi.fn(),
    }));
    await act(async () => {
      await result.current({
        id: '',
        slug: 'mixed',
        label: 'Mixed protocol gateway',
        endpoint: 'https://router.example/v1',
        authStyle: 'bearer',
        maskedKey: '',
        modelTransports: { 'response-model': 'responses', 'chat-model': 'chat_completions' },
      }, '', { skipProbe: true });
    });
    expect(api.flushCloudProviders).toHaveBeenCalledWith(expect.arrayContaining([
      expect.objectContaining({
        slug: 'mixed',
        model_transports: { 'response-model': 'responses', 'chat-model': 'chat_completions' },
      }),
    ]));
    expect(persist).toHaveBeenCalledWith(expect.objectContaining({
      cloudProviders: [expect.objectContaining({
        slug: 'mixed',
        modelTransports: { 'response-model': 'responses', 'chat-model': 'chat_completions' },
      })],
    }));
  });

  it('validates the provider CA before persisting a new key', async () => {
    api.flushCloudProviders.mockRejectedValue(new Error('invalid CA bundle'));
    const { result } = renderHook(() =>
      useCloudProviderEditorSubmit({
        editing: 'new',
        draft: EMPTY_SETTINGS,
        saved: EMPTY_SETTINGS,
        persist: vi.fn(),
        t: key => key,
        onDone: vi.fn(),
      })
    );

    await act(async () => {
      await expect(
        result.current(
          {
            id: '',
            slug: 'private-provider',
            label: 'Private provider',
            endpoint: 'https://provider.example/v1',
            authStyle: 'bearer',
            maskedKey: '',
            caCertPem: 'invalid PEM',
          },
          'new-secret'
        )
      ).rejects.toThrow('invalid CA bundle');
    });

    expect(api.flushCloudProviders).toHaveBeenCalledTimes(2);
    expect(api.flushCloudProviders).toHaveBeenCalledWith(
      expect.arrayContaining([
        expect.objectContaining({ slug: 'private-provider', ca_cert_pem: 'invalid PEM' }),
      ])
    );
    expect(api.flushCloudProviders).toHaveBeenNthCalledWith(2, []);
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(api.listProviderModels).not.toHaveBeenCalled();
  });

  it('restores the previous provider list when key storage fails', async () => {
    api.flushCloudProviders.mockResolvedValue(undefined);
    api.flushCloudProviders.mockResolvedValueOnce(undefined).mockResolvedValueOnce(undefined);
    api.setCloudProviderKey.mockRejectedValue(new Error('keyring locked'));
    const previous = {
      id: 'old-provider',
      slug: 'private-provider',
      label: 'Old provider',
      endpoint: 'https://old.example/v1',
      authStyle: 'bearer' as const,
      maskedKey: '••••old',
      caCertPem: 'old CA',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [previous] };
    const { result } = renderHook(() =>
      useCloudProviderEditorSubmit({
        editing: previous,
        draft: saved,
        saved,
        persist: vi.fn(),
        t: key => key,
        onDone: vi.fn(),
      })
    );

    await act(async () => {
      await expect(
        result.current(
          { ...previous, endpoint: 'https://new.example/v1', caCertPem: 'new CA' },
          'replacement-key'
        )
      ).rejects.toThrow('keyring locked');
    });

    expect(api.flushCloudProviders).toHaveBeenCalledTimes(2);
    expect(api.flushCloudProviders).toHaveBeenNthCalledWith(
      1,
      expect.arrayContaining([
        expect.objectContaining({ endpoint: 'https://new.example/v1', ca_cert_pem: 'new CA' }),
      ])
    );
    expect(api.flushCloudProviders).toHaveBeenNthCalledWith(
      2,
      expect.arrayContaining([
        expect.objectContaining({ endpoint: 'https://old.example/v1', ca_cert_pem: 'old CA' }),
      ])
    );
    expect(api.listProviderModels).not.toHaveBeenCalled();
  });

  it('restores the previous provider list when the eager flush fails', async () => {
    api.flushCloudProviders
      .mockRejectedValueOnce(new Error('settings write failed'))
      .mockResolvedValueOnce(undefined);
    const previous = {
      id: 'old-provider',
      slug: 'private-provider',
      label: 'Old provider',
      endpoint: 'https://old.example/v1',
      authStyle: 'bearer' as const,
      maskedKey: '••••old',
      caCertPem: 'old CA',
    };
    const saved = { ...EMPTY_SETTINGS, cloudProviders: [previous] };
    const { result } = renderHook(() =>
      useCloudProviderEditorSubmit({
        editing: previous,
        draft: saved,
        saved,
        persist: vi.fn(),
        t: key => key,
        onDone: vi.fn(),
      })
    );

    await act(async () => {
      await expect(
        result.current(
          { ...previous, endpoint: 'https://new.example/v1', caCertPem: 'new CA' },
          'replacement-key'
        )
      ).rejects.toThrow('settings write failed');
    });

    expect(api.flushCloudProviders).toHaveBeenCalledTimes(2);
    expect(api.flushCloudProviders).toHaveBeenNthCalledWith(
      2,
      expect.arrayContaining([
        expect.objectContaining({ endpoint: 'https://old.example/v1', ca_cert_pem: 'old CA' }),
      ])
    );
    expect(api.setCloudProviderKey).not.toHaveBeenCalled();
    expect(api.listProviderModels).not.toHaveBeenCalled();
  });
});
