import assert from 'node:assert/strict';
import test from 'node:test';
import { assertFullCatalogParity, parseVendorProviders } from '../check-cloud-provider-catalog-parity.mjs';

const ui = `export const BUILTIN_CLOUD_PROVIDERS = [
  { slug: 'existing', label: 'Existing', endpoint: 'https://existing.example/v1', authStyle: 'bearer', tone: TONE.sky },
  { slug: 'vendor', label: 'Vendor', endpoint: 'https://vendor.example/v1', authStyle: 'anthropic', tone: TONE.teal },
];`;
const host = `pub const EXTRA_CLOUD_PROVIDERS: &[BuiltinCloudProvider] = &[
  BuiltinCloudProvider { slug: "existing", label: "Existing", endpoint: "https://existing.example/v1", auth_style: AuthStyle::Bearer, },
];`;
const vendor = `pub const BUILTIN_CLOUD_PROVIDERS: &[BuiltinCloudProvider] = &[
  BuiltinCloudProvider { slug: "vendor", label: "Vendor", endpoint: "https://vendor.example/v1", auth_style: BuiltinAuthStyle::Anthropic, },
];`;

test('vendor provider parser uses canonical slugs, label, URL and auth', () => {
  assert.deepEqual(parseVendorProviders(vendor), [{
    slug: 'vendor', label: 'Vendor', endpoint: 'https://vendor.example/v1', auth: 'anthropic',
  }]);
});
test('checks all upstream vendor and host-defined providers against UI', () => {
  assert.deepEqual(assertFullCatalogParity(ui, host, vendor), {
    checked: 2, vendorTotal: 1, rustExtras: 1, frontendTotal: 2,
  });
});
test('a stale UI missing a newly pinned upstream provider fails', () => {
  const staleUI = ui.replace(/\s*\{ slug: 'vendor'[^\n]+\n/, '');
  assert.throws(() => assertFullCatalogParity(staleUI, host, vendor), /vendor.*missing.*frontend/i);
});
test('a UI-only provider no longer present in Rust fails', () => {
  const ghost = ui.replace('];', "  { slug: 'ghost', label: 'Ghost', endpoint: 'https://ghost.example/v1', authStyle: 'bearer' },\n];");
  assert.throws(() => assertFullCatalogParity(ghost, host, vendor), /ghost.*no matching.*Rust/i);
});
test('pinned upstream endpoint or auth changes are detected', () => {
  const endpoint = vendor.replace('https://vendor.example/v1', 'https://changed.example/v1');
  assert.throws(() => assertFullCatalogParity(ui, host, endpoint), /vendor.*endpoint mismatch/i);
  const auth = vendor.replace('BuiltinAuthStyle::Anthropic', 'BuiltinAuthStyle::Bearer');
  assert.throws(() => assertFullCatalogParity(ui, host, auth), /vendor.*auth mismatch/i);
});
test('host extras may not duplicate a vendor-owned provider slug', () => {
  const duplicate = host.replace('];', '  BuiltinCloudProvider { slug: "vendor", label: "Vendor", endpoint: "https://vendor.example/v1", auth_style: AuthStyle::Anthropic, },\n];');
  assert.throws(() => assertFullCatalogParity(ui, duplicate, vendor), /duplicate.*Rust.*vendor/i);
});
test('vendor URL security is validated even when matched by UI', () => {
  const unsafeUI = ui.replace('https://vendor.example/v1', 'http://remote.example/v1');
  const unsafeVendor = vendor.replace('https://vendor.example/v1', 'http://remote.example/v1');
  assert.throws(() => assertFullCatalogParity(unsafeUI, host, unsafeVendor), /vendor.*unsafe.*endpoint/i);
});
test('missing vendor catalog fails closed (no false full parity)', () => {
  assert.throws(() => assertFullCatalogParity(ui, host, 'missing upstream registry'), /could not find.*vendor/i);
});
