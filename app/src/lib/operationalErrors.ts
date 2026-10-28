/**
 * Operational diagnostics for failures outside the chat/provider error catalogs.
 *
 * Treat a backend rejection as data, not UI copy: keep a short, actionable
 * summary separate from a redacted technical detail. Each classifier requires
 * a producer-specific signal; a generic "failed" must never become an
 * incorrect port-conflict, permission, or Docker diagnosis.
 */
export type OperationalErrorKind =
  | 'storage_upgrade_required'
  | 'storage_migration_failed'
  | 'storage_unavailable'
  | 'credential_store_unavailable'
  | 'credential_save_failed'
  | 'local_memory_unavailable'
  | 'port_in_use'
  | 'unclassified';

export interface OperationalErrorPresentation {
  kind: OperationalErrorKind;
  summary: string;
  nextStep: string;
  /** Safe to display in an opt-in technical-details section. */
  technicalDetails: string | null;
  /** Whether retrying without an app update or intervention is plausible. */
  retryable: boolean;
}

function errorText(error: unknown): string {
  if (typeof error === 'string') return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === 'object') {
    const rejected = error as { error?: unknown; message?: unknown };
    if (typeof rejected.error === 'string') return rejected.error;
    if (typeof rejected.message === 'string') return rejected.message;
  }
  return '';
}

/** Basic defence in depth for free-form error strings crossing an IPC boundary. */
export function redactDiagnosticText(text: string): string {
  return text
    // Basic-auth URLs embed a username and password before the host.
    .replace(/([a-z][a-z0-9+.-]*:\/\/)[^\s/@]+@/gi, '$1[redacted]@')
    // Backend errors sometimes stringify a JSON credential-bearing response.
    .replace(
      /(["'])(api[_-]?key|access[_-]?token|refresh[_-]?token|id[_-]?token|client[_-]?secret|authorization|password|passwd|token|secret)\1(\s*:\s*)(["'])(.*?)\4/gi,
      (_whole, quote: string, name: string, separator: string, valueQuote: string) => {
        return `${quote}${name}${quote}${separator}${valueQuote}[redacted]${valueQuote}`;
      }
    )
    .replace(/(\bauthorization\s*:\s*(?:bearer|basic)\s+)[^\s,;]+/gi, '$1[redacted]')
    // Providers sometimes echo the submitted key in a prose error rather
    // than a structured field: "Incorrect API key provided: sk-...".
    .replace(
      /(\b(?:invalid|incorrect|expired)\s+(?:api[\s_-]*key|token)(?:\s+provided)?\s*:\s*)[^\s,;}"']+/gi,
      '$1[redacted]'
    )
    .replace(
      /((?:[?&]|\b)(?:api[_-]?key|access[_-]?token|refresh[_-]?token|id[_-]?token|client[_-]?secret|password|passwd|token|secret)\s*[:=]\s*)[^\s,;&"']+/gi,
      '$1[redacted]'
    )
    // Known SDK prefixes can appear outside an assignment or error field.
    .replace(/\bsk-[A-Za-z0-9_-]{3,}\b/g, '[redacted]');
}

function diagnosed(
  kind: OperationalErrorKind,
  summary: string,
  nextStep: string,
  technicalDetails: string | null,
  retryable: boolean
): OperationalErrorPresentation {
  return { kind, summary, nextStep, technicalDetails, retryable };
}

/**
 * Display-copy classifier shared by onboarding, settings, and recovery UIs.
 * Do not infer permissions, port conflicts or missing Docker from generic
 * "backend unavailable" messages. Keep full backend evidence in the logs.
 */
export function explainOperationalError(
  error: unknown,
  fallback = 'The operation did not complete.'
): OperationalErrorPresentation {
  const raw = errorText(error).trim();
  const detail = raw ? redactDiagnosticText(raw).slice(0, 2500) : null;
  const lower = raw.toLowerCase();

  if (lower.includes('legacy sqlite storage must be upgraded by the host')) {
    return diagnosed(
      'storage_upgrade_required',
      'This workspace uses an older database format.',
      'Update OpenHuman to a build that supports database migration. Do not delete the database or reset your workspace.',
      detail,
      false
    );
  }
  if (lower.includes('legacy sqlite migration') || lower.includes('mixed legacy and v2 tables')) {
    return diagnosed(
      'storage_migration_failed',
      'OpenHuman could not finish upgrading the workspace database.',
      'Preserve the existing database. Check migration details and available disk space before retrying with an updated build.',
      detail,
      false
    );
  }
  if (
    lower.includes('could not open the lock file') &&
    (lower.includes('secret') || lower.includes('keyring'))
  ) {
    return diagnosed(
      'credential_store_unavailable',
      'OpenHuman could not access encrypted credential storage.',
      'Close other OpenHuman processes, check permissions on the OpenHuman profile directory, and retry. Do not delete stored keys.',
      detail,
      true
    );
  }
  if (/failed to encrypt [\w.-]*api_key/.test(lower) || lower.includes('master key unavailable')) {
    return diagnosed(
      'credential_save_failed',
      'OpenHuman could not save the API key securely.',
      'Check that credential storage is available and writable, then retry. You can skip optional search setup for now.',
      detail,
      true
    );
  }
  if (lower.includes('docker is not ready')) {
    return diagnosed(
      'local_memory_unavailable',
      'Local memory cannot start because Docker is unavailable.',
      'Start Docker Desktop only if you selected on-computer memory; otherwise leave memory off and continue setup.',
      detail,
      true
    );
  }
  if (
    /(?:failed to bind|listen port|network port)/.test(lower) &&
    /(?:address already in use|os error 10048|port.*occupied)/.test(lower)
  ) {
    return diagnosed(
      'port_in_use',
      'OpenHuman could not bind its local network port.',
      'Close the process using that port or choose another port, then retry.',
      detail,
      true
    );
  }
  if (lower.includes('opening the configured storage backend')) {
    return diagnosed(
      'storage_unavailable',
      'OpenHuman could not open workspace storage.',
      'Review the detailed storage error and installed app version. Do not delete workspace data to resolve a generic startup failure.',
      detail,
      false
    );
  }

  return diagnosed(
    'unclassified',
    detail || fallback,
    'Review technical details and retry. If the problem persists, open Diagnostics and share a redacted log.',
    detail,
    true
  );
}
