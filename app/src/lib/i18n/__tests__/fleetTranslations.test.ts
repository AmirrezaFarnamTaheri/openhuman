import { describe, expect, it } from 'vitest';

import { fleetTranslationKeys, fleetTranslations } from '../fleetTranslations';
import { modelManagementTranslations } from '../modelManagement';

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

  it('translates every model-management message without silently using English', () => {
    const english = modelManagementTranslations.en;
    for (const [locale, messages] of Object.entries(modelManagementTranslations)) {
      expect(Object.keys(messages).sort(), locale).toEqual(Object.keys(english).sort());
      for (const [key, value] of Object.entries(messages)) {
        expect(value.trim(), `${locale}: ${key}`).toBeTruthy();
        // "vision" is also the French word; it is not an untranslated label.
        if (locale !== 'en' && !(locale === 'fr' && key === 'fleet.visionLabel')) {
          expect(value, `${locale}: ${key}`).not.toBe(english[key as keyof typeof english]);
        }
      }
    }
  });
  it('localizes suitability browsing and destructive-sync consent across supported locales', () => {
    const keys = [
      'settings.ai.feasibility.search',
      'settings.ai.feasibility.filter',
      'settings.ai.feasibility.more',
      'settings.ai.feasibility.costUnknown',
      'settings.ai.fleet.approveProviderSync',
      'settings.ai.fleet.approveMetadataRefresh',
    ];
    for (const [locale, messages] of Object.entries(fleetTranslations)) {
      for (const key of keys) {
        expect(messages[key]?.trim(), locale + ': ' + key).toBeTruthy();
      }
    }
    expect(fleetTranslations.es['settings.ai.feasibility.more']).not.toBe(
      fleetTranslations.en['settings.ai.feasibility.more']
    );
    expect(fleetTranslations.fr['settings.ai.feasibility.filter']).not.toBe(
      fleetTranslations.en['settings.ai.feasibility.filter']
    );
  });
});
