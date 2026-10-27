/**
 * The editor-side counterpart of Rust's `security::transport` bearer
 * endpoint boundary. Do not change the core policy or rewrite the URL:
 * reject unsafe input before a settings flush or credential write happens.
 */
export type ProviderEndpointIssue = 'invalid_url' | 'insecure_bearer';

export function providerEndpointIssue(
  endpoint: string,
  hasBearerCredential: boolean
): ProviderEndpointIssue | null {
  // WHATWG URL parsing silently discards tabs and newlines, so refuse
  // control characters before normalizing even the surrounding whitespace.
  if (/[\u0000-\u001F\u007F]/u.test(endpoint)) {
    return 'invalid_url';
  }
  const value = endpoint.trim();
  // The existing form can intentionally leave the endpoint unset; the
  // downstream provider configuration owns its default/required-url rules.
  if (!value) return null;
  let url: URL;
  try {
    url = new URL(value);
  } catch {
    return 'invalid_url';
  }
  if (
    !['http:', 'https:'].includes(url.protocol) ||
    !url.hostname ||
    url.username ||
    url.password ||
    url.search ||
    url.hash ||
    value.includes('?') ||
    value.includes('#')
  ) {
    return 'invalid_url';
  }
  if (hasBearerCredential && url.protocol === 'http:') {
    const host = url.hostname.toLowerCase();
    if (host !== 'localhost' && host !== '[::1]' && !/^127(?:\.\d{1,3}){3}$/.test(host)) {
      return 'insecure_bearer';
    }
  }
  return null;
}
