import { describe, expect, it } from 'vitest';

import { messageForApiError } from './apiError';

describe('messageForApiError', () => {
  it('uses the message from an Error', () => {
    expect(messageForApiError(new Error('Connection lost'), 'Try again.')).toBe('Connection lost');
  });

  it('uses the string error from an apiClient rejection', () => {
    expect(
      messageForApiError({ success: false, error: 'You have already reported this.' }, 'Try again.')
    ).toBe('You have already reported this.');
  });

  it('explains an outdated workspace without calling it a port conflict', () => {
    const displayed = messageForApiError(
      { success: false, error: 'legacy sqlite storage must be upgraded by the host' },
      'Try again.'
    );
    expect(displayed).toContain('older database format');
    expect(displayed).toContain('Do not delete');
    expect(displayed.toLowerCase()).not.toContain('port conflict');
  });

  it('explains an unavailable secret store and keeps remediation visible', () => {
    const displayed = messageForApiError(
      new Error('Secret-storage backend error: secret store: backend: could not open the lock file'),
      'Try again.'
    );
    expect(displayed).toContain('encrypted credential storage');
    expect(displayed).toContain('Close other OpenHuman processes');
  });

  it('redacts credential-bearing API failures before they reach any UI', () => {
    const displayed = messageForApiError(
      { error: 'failure: {"api_key":"private-key", "password":"hunter2"} token=oauth-secret' },
      'Try again.'
    );
    expect(displayed).not.toContain('private-key');
    expect(displayed).not.toContain('hunter2');
    expect(displayed).not.toContain('oauth-secret');
    expect(displayed).toContain('[redacted]');
  });

  it.each([
    new Error(''),
    { success: false, error: '' },
    { success: false, error: '   ' },
    { success: false, error: { reason: 'Not safe to display' } },
    { success: false },
    null,
  ])('uses the fallback for a blank or missing display message: %j', error => {
    expect(messageForApiError(error, 'Try again.')).toBe('Try again.');
  });
});
