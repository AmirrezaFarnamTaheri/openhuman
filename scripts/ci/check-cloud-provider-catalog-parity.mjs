/**
 * CI contract: all frontend presets must match the Rust host extensions plus
 * the pinned tinyinference vendor registry. Validate URLs and transport
 * identities before allowing a provider catalog change to ship.
 */
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

function capture(key, body, quote) {
  return new RegExp('\\b' + key + '\\s*:\\s*' + quote + '([^' + quote + '\\n]*)' + quote).exec(body)?.[1] ?? null;
}

function unique(rows, source) {
  const seen = new Set();
  for (const row of rows) {
    if (seen.has(row.slug)) throw new Error('duplicate ' + source + ' provider slug: ' + row.slug);
    seen.add(row.slug);
  }
  return rows;
}

export function parseFrontendProviders(source) {
  const section = /export const BUILTIN_CLOUD_PROVIDERS[^=]*=\s*\[([\s\S]*?)\s*\];/.exec(source)?.[1];
  if (!section) throw new Error('could not find frontend built-in provider array');
  const rows = [...section.matchAll(/\{\s*slug:\s*'([^']+)'([\s\S]*?)\}/g)].map(([, slug, tail]) => {
    const body = "slug: '" + slug + "' " + tail;
    const label = capture('label', body, "'");
    const endpoint = capture('endpoint', body, "'");
    const auth = capture('authStyle', body, "'");
    if (!label || !endpoint || !auth) throw new Error('incomplete frontend provider entry: ' + slug);
    return { slug, label, endpoint, auth };
  });
  if (!rows.length) throw new Error('could not find frontend provider entries');
  return unique(rows, 'frontend');
}

export function parseRustExtraProviders(source) {
  const section = /pub const EXTRA_CLOUD_PROVIDERS[^=]*=\s*&\[([\s\S]*?)\s*\];/.exec(source)?.[1];
  if (!section) throw new Error('could not find Rust extra provider array');
  const rows = [...section.matchAll(/BuiltinCloudProvider\s*\{([^}]*)\}/g)].map(([, body]) => {
    const slug = capture('slug', body, '"');
    const label = capture('label', body, '"');
    const endpoint = capture('endpoint', body, '"');
    const style = /auth_style\s*:\s*AuthStyle::(Bearer|Anthropic|None|OpenhumanJwt)/.exec(body)?.[1];
    if (!slug || !label || !endpoint || !style) throw new Error('incomplete Rust provider entry: ' + (slug ?? '?'));
    const auth = { Bearer: 'bearer', Anthropic: 'anthropic', None: 'none', OpenhumanJwt: 'openhuman_jwt' }[style];
    return { slug, label, endpoint, auth };
  });
  if (!rows.length) throw new Error('could not find Rust extra provider entries');
  return unique(rows, 'Rust');
}

/** Only HTTPS or HTTP to loopback is allowed in trusted built-in defaults. */
export function isSafeCloudEndpoint(raw) {
  if (typeof raw !== 'string' || !raw || raw !== raw.trim()) return false;
  let url;
  try { url = new URL(raw); } catch { return false; }
  if (!url.hostname || url.username || url.password || url.search || url.hash) return false;
  if (url.protocol === 'https:') return true;
  if (url.protocol !== 'http:') return false;
  const host = url.hostname.toLowerCase();
  return host === 'localhost' || host === '[::1]' || /^127(?:\.\d{1,3}){3}$/.test(host);
}

export function assertCatalogParity(frontendSource, rustSource) {
  const frontend = new Map(parseFrontendProviders(frontendSource).map(row => [row.slug, row]));
  // The Rust extras do not list provider-neutral vendor presets. Every UI
  // builtin still receives user credentials and therefore needs a safe URL.
  for (const entry of frontend.values()) {
    if (!isSafeCloudEndpoint(entry.endpoint)) {
      throw new Error(entry.slug + ' has an unsafe built-in endpoint');
    }
  }
  const rust = parseRustExtraProviders(rustSource);
  for (const entry of rust) {
    if (!isSafeCloudEndpoint(entry.endpoint)) throw new Error(entry.slug + ' has an unsafe built-in endpoint');
    const candidate = frontend.get(entry.slug);
    if (!candidate) throw new Error(entry.slug + ' missing from frontend built-in catalog');
    for (const field of ['endpoint', 'auth', 'label']) {
      if (candidate[field] !== entry[field]) {
        throw new Error(entry.slug + ' ' + field + ' mismatch: frontend=' + candidate[field] + ', Rust=' + entry[field]);
      }
    }
  }
  return { checked: rust.length, frontendTotal: frontend.size };
}

/**
 * Parse the *pinned* tin yinference provider catalog, not a live web listing.
 * With the host extras this is the actual Rust backend registry that the
 * frontend must be able to configure. CI checks out nested git submodules.
 */
export function parseVendorProviders(source) {
  const section = /pub const BUILTIN_CLOUD_PROVIDERS[^=]*=\s*&\[([\s\S]*?)\s*\];/.exec(source)?.[1];
  if (!section) throw new Error('could not find vendor built-in provider registry');
  const rows = [...section.matchAll(/BuiltinCloudProvider\s*\{([^}]*)\}/g)].map(([, body]) => {
    const slug = capture('slug', body, '"');
    const label = capture('label', body, '"');
    const endpoint = capture('endpoint', body, '"');
    const style = /auth_style\s*:\s*BuiltinAuthStyle::(Bearer|Anthropic)/.exec(body)?.[1];
    if (!slug || !label || !endpoint || !style) {
      throw new Error('incomplete vendor provider entry: ' + (slug ?? '?'));
    }
    return { slug, label, endpoint, auth: style.toLowerCase() };
  });
  if (!rows.length) throw new Error('empty vendor provider registry');
  return unique(rows, 'vendor Rust');
}

/**
 * Full parity closes the UI-only blind spot in assertCatalogParity:
 * vendor-builtins + host extras must equal the UI builtins, byte-for-byte on
 * identity and transport fields. A new upstream preset cannot silently remain
 * missing in the UI; a removed upstream preset cannot keep a broken UI tile.
 */
export function assertFullCatalogParity(frontendSource, hostSource, vendorSource) {
  const frontendRows = parseFrontendProviders(frontendSource);
  const frontend = new Map(frontendRows.map(row => [row.slug, row]));
  for (const entry of frontendRows) {
    if (!isSafeCloudEndpoint(entry.endpoint)) throw new Error(entry.slug + ' has an unsafe built-in endpoint');
  }
  const extras = parseRustExtraProviders(hostSource);
  const vendor = parseVendorProviders(vendorSource);
  const backend = new Map();
  for (const entry of [...vendor, ...extras]) {
    if (backend.has(entry.slug)) throw new Error('duplicate Rust provider slug across vendor and extras: ' + entry.slug);
    if (!isSafeCloudEndpoint(entry.endpoint)) throw new Error(entry.slug + ' has an unsafe built-in endpoint');
    backend.set(entry.slug, entry);
    const candidate = frontend.get(entry.slug);
    if (!candidate) throw new Error(entry.slug + ' missing from frontend built-in catalog');
    for (const field of ['endpoint', 'auth', 'label']) {
      if (candidate[field] !== entry[field]) {
        throw new Error(entry.slug + ' ' + field + ' mismatch: frontend=' + candidate[field] + ', Rust=' + entry[field]);
      }
    }
  }
  for (const entry of frontendRows) {
    if (!backend.has(entry.slug)) throw new Error(entry.slug + ' has no matching Rust provider');
  }
  return { checked: backend.size, vendorTotal: vendor.length, rustExtras: extras.length, frontendTotal: frontend.size };
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) {
  try {
    // This file is scripts/ci/*.mjs (two directories below repository root).
    const root = resolve(fileURLToPath(new URL('../../', import.meta.url)));
    const frontend = readFileSync(resolve(root, 'app/src/components/settings/panels/builtinCloudProviders.ts'), 'utf8');
    const rust = readFileSync(resolve(root, 'crates/openhuman-core/src/config/schema/cloud_providers.rs'), 'utf8');
    const vendor = readFileSync(
      resolve(root, 'vendor/tinyagents/vendor/tinyinference/crates/tinyinference-providers/src/builtin.rs'),
      'utf8'
    );
    const result = assertFullCatalogParity(frontend, rust, vendor);
    console.log('provider catalog parity OK: ' + result.checked + ' Rust providers (' + result.vendorTotal + ' upstream + ' + result.rustExtras + ' extras) agree with UI');
  } catch (error) {
    console.error('provider catalog parity failed: ' + error.message);
    process.exitCode = 1;
  }
}
