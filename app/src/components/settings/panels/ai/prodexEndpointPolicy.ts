/** UI counterpart of Rust's prodex_gateway_endpoint_is_safe.
 * The Rust factory remains authoritative before any credential lookup.
 * Alternate image/model endpoints receive the same virtual bearer key.
 */
export function validProdexDestination(raw: string): boolean {
  if (!raw.trim() || raw.trim() !== raw) return false;
  let url: URL;
  try {
    url = new URL(raw);
  } catch {
    return false;
  }
  if (!url.hostname || url.username || url.password || url.search || url.hash) return false;
  if (url.protocol === 'https:') return true;
  if (url.protocol !== 'http:') return false;
  const host = url.hostname.toLowerCase();
  if (host === 'localhost' || host === '[::1]') return true;
  const octets = host.split('.');
  return (
    octets.length === 4 &&
    octets[0] === '127' &&
    octets.every(part => /^\d{1,3}$/.test(part) && Number(part) <= 255)
  );
}
