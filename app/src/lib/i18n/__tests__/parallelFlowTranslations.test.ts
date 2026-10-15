import { describe, expect, it } from 'vitest';

import { parallelFlowTranslationKeys, parallelFlowTranslations } from '../parallelFlowTranslations';

describe('native scatter/gather localization', () => {
  it('covers every new native parallel-workflow control in all sixteen locales', () => {
    expect(Object.keys(parallelFlowTranslations)).toHaveLength(16);
    for (const [locale, labels] of Object.entries(parallelFlowTranslations)) {
      expect(Object.keys(labels).sort(), locale).toEqual([...parallelFlowTranslationKeys].sort());
      for (const value of Object.values(labels)) {
        expect(value.trim().length, locale).toBeGreaterThan(0);
      }
    }
  });
});
