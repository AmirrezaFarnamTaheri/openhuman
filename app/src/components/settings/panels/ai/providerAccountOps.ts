/** Named API-key accounts have their own profiles. OAuth and delegated
 * gateways have separate authorization and rotation semantics.
 */
export function validateNamedProviderAccount(raw: string): string {
  const value = raw.trim();
  if (
    !/^[A-Za-z0-9][A-Za-z0-9_-]{0,47}$/.test(value) ||
    ['default', 'oauth'].includes(value.toLowerCase())
  ) {
    throw new Error(
      'Use 1-48 letters, numbers, dashes or underscores; default/oauth are reserved.'
    );
  }
  return value;
}
export function validateAccountApiKey(raw: string): string {
  const value = raw.trim();
  if (!value || value.length > 4096 || /[\s,;]/.test(value)) {
    throw new Error('Provide exactly one nonempty provider API key.');
  }
  return value;
}
export function accountFleetEligible(slug: string, authStyle: string): boolean {
  return (
    slug !== 'prodex' &&
    slug !== 'openhuman' &&
    (authStyle === 'bearer' || authStyle === 'anthropic')
  );
}
