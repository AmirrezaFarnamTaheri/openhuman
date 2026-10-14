import { describe, expect, it } from 'vitest';

import { FLEET_MODEL_PRESETS } from '../modelFleetCatalog';
import {
  applyModelBulkPatch,
  applyProviderBulkPatch,
  insertFleetKeysAtSelection,
  parseFleetApiKeys,
  parseModelImageEndpoints,
  syncConfiguredProviderPresets,
  syncModelPresets,
  updateSelectedGroupSettings,
  validModelFleetEndpoint,
  validModelFleetImageEndpoint,
} from '../modelFleetOps';

const initial = {
  id: 'handwritten',
  provider: 'custom',
  context_window: 24_000,
  cost_per_1m_output: 2,
  vision: false,
};

describe('model-fleet preset management', () => {
  it('ports all Penguin presets without duplicate provider/model identities', () => {
    expect(FLEET_MODEL_PRESETS.length).toBe(127);
    const ids = new Set(FLEET_MODEL_PRESETS.map(p => p.provider + '\0' + p.id));
    expect(ids.size).toBe(127);
    expect(FLEET_MODEL_PRESETS.some(p => p.provider === 'tokendance')).toBe(true);
    expect(FLEET_MODEL_PRESETS.some(p => p.provider === 'qwen-token-plan')).toBe(true);
  });

  it('parses credential groups and avoids duplicate token entries', () => {
    expect(parseFleetApiKeys('  a, b; a\nc\r\n')).toEqual(['a', 'b', 'c']);
    expect(parseFleetApiKeys('one')).toEqual(['one']);
    expect(parseFleetApiKeys(' , ;\n')).toEqual([]);
  });

  it('preserves an occupied key when additional keys are pasted at a cursor', () => {
    const rows = ['original-key', 'other-key'];
    expect(insertFleetKeysAtSelection(rows, 0, 'pasted-1\npasted-2', 4, 4)).toEqual([
      'original-key',
      'pasted-1',
      'pasted-2',
      'other-key',
    ]);
    expect(rows).toEqual(['original-key', 'other-key']);
  });

  it('replaces the selected key when its whole value is selected', () => {
    expect(insertFleetKeysAtSelection(['old', 'other'], 0, 'new-a;new-b', 0, 3)).toEqual([
      'new-a',
      'new-b',
      'other',
    ]);
    expect(insertFleetKeysAtSelection(['', 'other'], 0, 'new-a,new-b', 0, 0)).toEqual([
      'new-a',
      'new-b',
      'other',
    ]);
  });

  it('never silently truncates credential batches over the row limit', () => {
    const full = Array.from({ length: 128 }, (_, i) => 'key-' + i);
    expect(() => insertFleetKeysAtSelection(full, 0, 'additional-a\nadditional-b', 0, 0)).toThrow(
      /128/
    );
    expect(full).toHaveLength(128);
  });

  it('syncs catalog fields without removing custom models and is idempotent', () => {
    const { models, added, updated } = syncModelPresets([initial]);
    expect(added).toBe(127);
    expect(updated).toBe(0);
    expect(models[0]).toEqual(initial);
    const next = syncModelPresets(models);
    expect(next.added).toBe(0);
    expect(next.updated).toBe(0);
    expect(next.models).toEqual(models);
  });

  it('refreshes built-in model metadata but preserves unrecognized entries', () => {
    const preset = FLEET_MODEL_PRESETS[0];
    const old = {
      id: preset.id,
      provider: preset.provider,
      vision: !preset.vision,
      cost_per_1m_output: 100,
      context_window: 1,
    };
    const result = syncModelPresets([initial, old]);
    expect(result.models.find(m => m.id === old.id && m.provider === old.provider)?.vision).toBe(
      preset.vision
    );
    expect(result.updated).toBe(1);
    expect(result.models[0]).toEqual(initial);
  });

  it('bulk edits only selected entries without changing other models', () => {
    const out = applyModelBulkPatch(
      [initial, { ...initial, id: 'other' }],
      new Set(['custom\0handwritten']),
      { vision: true, context_window: 123 }
    );
    expect(out[0].vision).toBe(true);
    expect(out[0].context_window).toBe(123);
    expect(out[1].vision).toBe(false);
  });

  it('preserves credentials and user-defined provider endpoints when syncing known presets', () => {
    const rows = [
      {
        id: 'p1',
        slug: 'openrouter',
        label: 'My OpenRouter',
        endpoint: 'https://old.example/v1',
        auth_style: 'bearer' as const,
        image_endpoint: 'https://image.example/v1',
        model_image_endpoints: { vision: 'https://vision.example/v1' },
      },
    ];
    const { providers, updated } = syncConfiguredProviderPresets(rows);
    expect(updated).toBe(1);
    expect(providers[0].label).toBe('My OpenRouter');
    expect(providers[0].image_endpoint).toBe(rows[0].image_endpoint);
    expect(providers[0].model_image_endpoints).toEqual(rows[0].model_image_endpoints);
    expect(providers[0].endpoint).toBe('https://openrouter.ai/api/v1');
  });

  it('validates keyed image model endpoints and rejects duplicate model ids', () => {
    expect(
      parseModelImageEndpoints(
        'model-x = https://vision.example/v1\nmodel-y = http://localhost:8000/v1'
      )
    ).toEqual({ 'model-x': 'https://vision.example/v1', 'model-y': 'http://localhost:8000/v1' });
    expect(() => parseModelImageEndpoints('x = https://a.test\nx = https://b.test')).toThrow(
      /Duplicate/
    );
    expect(() => parseModelImageEndpoints('x = ftp://a.test')).toThrow();
    expect(validModelFleetEndpoint('https://user:secret@host.test/v1')).toBe(false);
    expect(validModelFleetEndpoint('https://api.example.test/v1?token=abc')).toBe(false);
    expect(validModelFleetEndpoint('https://api.example.test/v1#fragment')).toBe(false);
    expect(validModelFleetEndpoint('http://127.0.0.1:8800/v1')).toBe(true);
  });

  it('blocks remote cleartext image endpoint overrides before credentials can be sent', () => {
    expect(validModelFleetEndpoint('http://example.net/v1')).toBe(true);
    for (const url of [
      'http://example.net/v1',
      'http://10.10.0.5:8000/v1',
      'http://169.254.169.254/latest/meta-data',
      'http://localhost.evil.test/v1',
      'https://user:secret@images.example/v1',
    ]) {
      expect(validModelFleetImageEndpoint(url)).toBe(false);
      expect(() => parseModelImageEndpoints('vision = ' + url)).toThrow();
    }
    for (const url of [
      'https://images.example/v1',
      'http://localhost:4000/v1',
      'http://127.0.0.2:4000/v1',
      'http://[::1]:4000/v1',
    ]) {
      expect(validModelFleetImageEndpoint(url)).toBe(true);
      expect(parseModelImageEndpoints('vision = ' + url)).toEqual({ vision: url });
    }
    expect(validModelFleetImageEndpoint('')).toBe(true);
  });

  it('changes a key rotation strategy without clearing the existing image endpoint', () => {
    const providers = [
      {
        slug: 'openai',
        imageEndpoint: 'https://vision.example.com/v1',
        rotationStrategy: 'least_busy' as const,
        modelImageEndpoints: { 'vision-model': 'https://vision-model.example.com/v1' },
      },
      {
        slug: 'anthropic',
        imageEndpoint: 'https://other.example.com/v1',
        rotationStrategy: 'round_robin' as const,
      },
    ];
    const selected = new Set(['openai']);
    const changed = updateSelectedGroupSettings(providers, selected, {
      rotationStrategy: 'priority_weighted',
    });
    expect(changed[0].rotationStrategy).toBe('priority_weighted');
    expect(changed[0].imageEndpoint).toBe('https://vision.example.com/v1');
    expect(changed[0].modelImageEndpoints).toEqual(providers[0].modelImageEndpoints);
    expect(changed[1]).toEqual(providers[1]);
    const cleared = updateSelectedGroupSettings(changed, selected, { imageEndpoint: '' });
    expect(cleared[0].imageEndpoint).toBe('');
    expect(cleared[0].rotationStrategy).toBe('priority_weighted');
  });

  it('bulk image-endpoint edit preserves mapped model endpoints', () => {
    const rows = [
      {
        id: 'one',
        slug: 'a',
        endpoint: 'https://a.test/v1',
        auth_style: 'bearer' as const,
        image_endpoint: 'https://old.test/v1',
        model_image_endpoints: { x: 'https://x.test/v1' },
      },
      { id: 'two', slug: 'b', endpoint: 'https://b.test/v1', auth_style: 'bearer' as const },
    ];
    const result = applyProviderBulkPatch(rows, new Set(['a']), {
      image_endpoint: 'https://new.test/v1',
      rotation_strategy: 'least_busy',
    });
    expect(result[0].image_endpoint).toBe('https://new.test/v1');
    expect(result[0].model_image_endpoints).toEqual(rows[0].model_image_endpoints);
    expect(result[1]).toEqual(rows[1]);
  });
});
