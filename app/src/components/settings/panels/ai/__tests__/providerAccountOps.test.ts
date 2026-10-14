import { describe, expect, it } from 'vitest';

import {
  accountFleetEligible,
  validateAccountApiKey,
  validateNamedProviderAccount,
} from '../providerAccountOps';

describe('named account contract', () => {
  it('rejects reserved, traversal and oversized names', () => {
    expect(validateNamedProviderAccount(' work-2 ')).toBe('work-2');
    for (const value of ['default', 'oauth', '../other', 'x'.repeat(49)]) {
      expect(() => validateNamedProviderAccount(value)).toThrow();
    }
  });
  it('requires exactly one key per account', () => {
    expect(validateAccountApiKey(' sk-safe ')).toBe('sk-safe');
    for (const value of ['', 'one,two', 'one;two', 'one\ntwo']) {
      expect(() => validateAccountApiKey(value)).toThrow();
    }
  });
  it('supports provider BYOK while preserving delegated authorities', () => {
    expect(accountFleetEligible('google', 'bearer')).toBe(true);
    expect(accountFleetEligible('anthropic', 'anthropic')).toBe(true);
    expect(accountFleetEligible('openai', 'bearer')).toBe(true);
    expect(accountFleetEligible('prodex', 'bearer')).toBe(false);
    expect(accountFleetEligible('openhuman', 'openhuman_jwt')).toBe(false);
  });
});
