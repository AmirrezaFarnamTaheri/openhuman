import { describe, expect, it } from 'vitest';

import { explainOperationalError } from './operationalErrors';

describe('explainOperationalError', () => {
  it('diagnoses an old SQLite layout without blaming the RPC port', () => {
    const diagnosed = explainOperationalError(
      new Error(
        'opening configured backend: invalid input: legacy sqlite storage must be upgraded by the host'
      )
    );
    expect(diagnosed.kind).toBe('storage_upgrade_required');
    expect(diagnosed.summary).toMatch(/older database/i);
    expect(diagnosed.nextStep).toMatch(/update OpenHuman/i);
    expect(diagnosed.nextStep).toMatch(/do not delete/i);
    expect(diagnosed.summary).not.toMatch(/port/i);
  });

  it('separates a credential lock failure from a network failure', () => {
    const diagnosed = explainOperationalError(
      new Error('Secret-storage backend error: secret store: backend: could not open the lock file')
    );
    expect(diagnosed.kind).toBe('credential_store_unavailable');
    expect(diagnosed.summary).toMatch(/credential storage/i);
    expect(diagnosed.nextStep).toMatch(/permissions/i);
    expect(diagnosed.nextStep).toMatch(/do not delete/i);
  });

  it('makes key-encryption failures actionable without assuming a lock error', () => {
    const diagnosed = explainOperationalError(
      new Error('Failed to encrypt search.tinyfish.api_key')
    );
    expect(diagnosed.kind).toBe('credential_save_failed');
    expect(diagnosed.summary).toMatch(/save.*key/i);
    expect(diagnosed.nextStep).toMatch(/credential storage/i);
  });

  it('treats Docker as an optional-memory dependency, not a core startup failure', () => {
    const diagnosed = explainOperationalError(
      new Error(
        'unavailable: Docker is not ready. Start Docker Desktop or Docker Engine and retry.'
      )
    );
    expect(diagnosed.kind).toBe('local_memory_unavailable');
    expect(diagnosed.nextStep).toMatch(/memory off/i);
  });

  it('passes through useful generic API errors but never stringifies arbitrary objects', () => {
    expect(explainOperationalError({ error: 'The provider returned 403' }).summary).toContain('403');
    expect(explainOperationalError({ error: { secrets: 'unsafe' } }).technicalDetails).toBeNull();
    expect(explainOperationalError({}).summary).toBe('The operation did not complete.');
  });

  it('redacts tokens and passwords from both visible and technical text', () => {
    const diagnosed = explainOperationalError(
      new Error(
        'Request failed: Authorization: Bearer abcdef123; password=my-secret; https://test.example?access_token=token123'
      )
    );
    expect(diagnosed.summary + diagnosed.technicalDetails).not.toContain('abcdef123');
    expect(diagnosed.summary + diagnosed.technicalDetails).not.toContain('my-secret');
    expect(diagnosed.summary + diagnosed.technicalDetails).not.toContain('token123');
  });

  it('does not expose structured credentials or URL userinfo in operational errors', () => {
    const response = explainOperationalError(
      new Error(
        'Remote error: {"api_key":"sk-secret-123","password":"pw-123",' +
          '"Authorization":"Bearer token-xyz","refreshToken":"refresh-123"}; ' +
          'https://user:pw-789@service.example/api?token=tkn-123'
      )
    );
    const displayed = response.summary + response.technicalDetails;
    for (const secret of ['sk-secret-123', 'pw-123', 'token-xyz', 'refresh-123', 'pw-789', 'tkn-123']) {
      expect(displayed).not.toContain(secret);
    }
    expect(displayed).toContain('[redacted]');
  });

  it('redacts echoed provider keys and arbitrarily shaped invalid-token values', () => {
    const error = explainOperationalError(
      new Error('Incorrect API key provided: sk-abc; Invalid token: FIXTURE-not-a-real-key-000000')
    );
    expect(error.summary).not.toContain('sk-abc');
    expect(error.summary).not.toContain('FIXTURE-not-a-real-key-000000');
    expect(error.technicalDetails).not.toContain('sk-abc');
    expect(error.technicalDetails).not.toContain('FIXTURE-not-a-real-key-000000');
  });

  it('does not infer a port conflict from an unrelated SQLite failure', () => {
    expect(
      explainOperationalError(new Error('opening the configured storage backend')).kind
    ).toBe('storage_unavailable');
  });

  it('classifies an actual address-in-use error distinctly', () => {
    const diagnosed = explainOperationalError(
      new Error('failed to bind 127.0.0.1:7788: Address already in use')
    );
    expect(diagnosed.kind).toBe('port_in_use');
    expect(diagnosed.nextStep).toMatch(/port/i);
  });
});
