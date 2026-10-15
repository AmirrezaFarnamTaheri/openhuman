import { describe, expect, it } from 'vitest';
import { fleetTranslationKeys, fleetTranslations } from '../fleetTranslations';

describe('provider fleet localization', () => {
  it('provides every provider-control translation in each supported locale', () => {
    for (const [locale, messages] of Object.entries(fleetTranslations)) {
      expect(Object.keys(messages).sort(), locale).toEqual([...fleetTranslationKeys].sort());
      for (const value of Object.values(messages)) {
        expect(value.trim().length, locale).toBeGreaterThan(0);
      }
    }
    expect(Object.keys(fleetTranslations)).toHaveLength(16);
  });

  it('keeps stable model protocol names untranslated', () => {
    for (const messages of Object.values(fleetTranslations)) {
      expect(messages['settings.ai.modelTransportOverridesHelp']).toContain('Responses');
      expect(messages['settings.ai.modelTransportOverridesHelp']).toContain('Auto');
    }
  });
});
