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
  const value = endpoint.trim();
  // The existing form can intentionally leave the endpoint unset; the
  // downstream provider configuration owns its default/required-url rules.
  if (!value) return null;
  // WHATWG URL parsing silently discards tabs and newlines, so reject them
  // before parsing. Keep user input rather than normalizing a different host.
  if (Array.from(value).some(char => {
    const code = char.charCodeAt(0);
    return code <= 31 || code === 127;
  })) {
    return 'invalid_url';
  }
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
