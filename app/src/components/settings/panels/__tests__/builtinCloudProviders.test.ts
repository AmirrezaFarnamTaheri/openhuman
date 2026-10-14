import { describe, expect, it } from 'vitest';

import {
  authStyleForBuiltinCloudProvider,
  BUILTIN_CLOUD_PROVIDER_SLUGS,
  BUILTIN_CLOUD_PROVIDERS,
  defaultEndpointForBuiltinCloudProvider,
} from '../builtinCloudProviders';

describe('builtinCloudProviders', () => {
  it('keeps built-in provider slugs unique', () => {
    expect(new Set(BUILTIN_CLOUD_PROVIDER_SLUGS).size).toBe(BUILTIN_CLOUD_PROVIDER_SLUGS.length);
  });

  it.each([
    ['gmi', 'https://api.gmi-serving.com/v1', 'bearer'],
    ['groq', 'https://api.groq.com/openai/v1', 'bearer'],
    ['deepseek', 'https://api.deepseek.com/v1', 'bearer'],
    ['minimax', 'https://api.minimax.io/v1', 'bearer'],
    ['sumopod', 'https://ai.sumopod.com/v1', 'bearer'],
    ['modelscope', 'https://api-inference.modelscope.cn/v1', 'bearer'],
    ['tokendance', 'https://tokendance.space/gateway/v1', 'bearer'],
    ['siliconflow', 'https://api.siliconflow.cn/v1', 'bearer'],
    ['qwen-pay-as-you-go', 'https://dashscope.aliyuncs.com/compatible-mode/v1', 'bearer'],
    [
      'qwen-token-plan',
      'https://token-plan.cn-beijing.maas.aliyuncs.com/compatible-mode/v1',
      'bearer',
    ],
    ['llmtr', 'https://llmtr.com/v1', 'bearer'],
    ['9router', 'http://127.0.0.1:20128/v1', 'bearer'],
    ['prodex', 'http://127.0.0.1:4000/v1', 'bearer'],
  ] as const)('maps %s to its endpoint and auth style', (slug, endpoint, authStyle) => {
    expect(defaultEndpointForBuiltinCloudProvider(slug)).toBe(endpoint);
    expect(authStyleForBuiltinCloudProvider(slug)).toBe(authStyle);
  });


  it.each([
    ['aionlabs', 'https://api.aionlabs.ai/v1'],
    ['sambanova', 'https://api.sambanova.ai/v1'],
    ['ovhcloud', 'https://oai.endpoints.kepler.ai.cloud.ovh.net/v1'],
    ['nebius', 'https://api.tokenfactory.nebius.com/v1'],
    ['llm7', 'https://api.llm7.io/v1'],
    ['synthetic', 'https://api.synthetic.new/openai/v1'],
  ] as const)('registers verified peer endpoint %s with bearer authentication', (slug, endpoint) => {
    expect(defaultEndpointForBuiltinCloudProvider(slug)).toBe(endpoint);
    expect(authStyleForBuiltinCloudProvider(slug)).toBe('bearer');
  });

  it('treats a local 9Router gateway as an optional external integration, not a saved account', () => {
    const router = BUILTIN_CLOUD_PROVIDERS.find(provider => provider.slug === '9router');
    expect(router).toMatchObject({
      endpoint: 'http://127.0.0.1:20128/v1',
      authStyle: 'bearer',
      keyPlaceholder: '9Router inference API key',
    });
    expect(router).not.toHaveProperty('defaultApiKey');
    expect(router?.endpoint).not.toMatch(/0\.0\.0\.0/);
  });

  it('prompts for a least-privilege inference key, not an administrator token', () => {
    const gateway = BUILTIN_CLOUD_PROVIDERS.find(provider => provider.slug === 'prodex');
    expect(gateway?.keyPlaceholder).toBe('Inference-scoped virtual key');
  });

  it('contains the full phase one provider set', () => {
    expect(BUILTIN_CLOUD_PROVIDERS.map(provider => provider.slug)).toEqual(
      expect.arrayContaining([
        'groq',
        'mistral',
        'deepseek',
        'together',
        'google',
        'cerebras',
        'xai',
        'huggingface',
        'nvidia',
        'zai',
        'minimax',
        'stepfun',
        'kilocode',
        'deepinfra',
        'novita',
        'venice',
        'vercel-ai-gateway',
        'sumopod',
        'modelscope',
        'llmtr',
        'prodex',
      ])
    );
  });
});
