import { explainOperationalError } from '../lib/operationalErrors';

/**
 * Return actionable, redacted display copy for API and IPC rejections.
 *
 * Both apiClient's { success, error } objects and native Error instances are
 * handled by the same narrow classifier. Unknown objects do not get coerced
 * to "[object Object]", while known storage/credential errors retain a
 * concrete next action. Full underlying evidence belongs in redacted logs.
 */
export function messageForApiError(error: unknown, fallback: string): string {
  const presentation = explainOperationalError(error, fallback);
  return presentation.kind === 'unclassified'
    ? presentation.summary
    : `${presentation.summary} ${presentation.nextStep}`;
}
