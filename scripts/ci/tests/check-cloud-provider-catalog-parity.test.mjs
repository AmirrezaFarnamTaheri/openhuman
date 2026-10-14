import { test } from 'node:test';
import assert from 'node:assert/strict';
import { assertCatalogParity, parseFrontendProviders, parseRustExtraProviders } from '../check-cloud-provider-catalog-parity.mjs';

const front = `export const BUILTIN_CLOUD_PROVIDERS = [
  { slug: 'one', label: 'One', endpoint: 'https://one.example/v1', authStyle: 'bearer', tone: TONE.sky },
  { slug: 'two', label: 'Two', endpoint: 'https://two.example/v1', authStyle: 'anthropic', tone: TONE.rose },
];`;
const back = `pub const EXTRA_CLOUD_PROVIDERS: &[BuiltinCloudProvider] = &[
  BuiltinCloudProvider { slug: "one", label: "One", endpoint: "https://one.example/v1", auth_style: AuthStyle::Bearer, },
  BuiltinCloudProvider { slug: "two", label: "Two", endpoint: "https://two.example/v1", auth_style: AuthStyle::Anthropic, },
];
pub static BUILTIN_CLOUD_PROVIDERS: LazyLock<Vec<BuiltinCloudProvider>> = LazyLock::new(|| {});`;

test('matching extra presets agree with frontend by slug, endpoint and authentication', () => {
  assert.deepEqual(parseFrontendProviders(front).map(x=>x.slug), ['one','two']);
  assert.deepEqual(parseRustExtraProviders(back).map(x=>x.slug), ['one','two']);
  assert.doesNotThrow(() => assertCatalogParity(front, back));
});

test('a Rust-only extra provider is a contract violation', () => {
  assert.throws(() => assertCatalogParity(front.replace(/\s*\{ slug: 'two'[^\n]+\n/,''), back), /two.*missing from.*frontend/i);
});

test('endpoint drift is detected before a user configures a broken route', () => {
  assert.throws(() => assertCatalogParity(front.replace('https://one.example/v1','https://wrong.example/v1'), back), /one.*endpoint mismatch/i);
});

test('authentication style drift is detected', () => {
  assert.throws(() => assertCatalogParity(front.replace("authStyle: 'bearer'", "authStyle: 'anthropic'"), back), /one.*auth mismatch/i);
});

test('duplicate slugs in either catalog are rejected', () => {
  assert.throws(() => assertCatalogParity(front.replace('];',"  { slug: 'one', label: 'Other', endpoint: 'https://one.example/v1', authStyle: 'bearer' },\n];"), back), /duplicate.*frontend.*one/i);
  assert.throws(() => assertCatalogParity(front, back.replace('];', '  BuiltinCloudProvider { slug: "one", label: "One", endpoint: "https://one.example/v1", auth_style: AuthStyle::Bearer, },\n];')), /duplicate.*Rust.*one/i);
});

test('a missing or malformed registry fails closed', () => {
  assert.throws(() => assertCatalogParity('export const BUILTIN_CLOUD_PROVIDERS = []', back), /could not find.*frontend/i);
  assert.throws(() => assertCatalogParity(front, 'pub const EXTRA_CLOUD_PROVIDERS: &[BuiltinCloudProvider] = &[];'), /could not find.*Rust/i);
});

test('rejects insecure remote HTTP or URL-carried secrets in built-in routes', () => {
  for (const endpoint of [
    'http://untrusted.example/v1',
    'https://key:secret@one.example/v1',
    'https://one.example/v1?key=secret',
    'https://one.example/v1#token',
    'file:///secrets',
  ]) {
    const candidate = front.replace('https://one.example/v1', endpoint);
    const paired = back.replace('https://one.example/v1', endpoint);
    assert.throws(() => assertCatalogParity(candidate, paired), /unsafe.*endpoint/i);
  }
});

test('permits a credential-free loopback gateway for separately scoped keys', () => {
  const frontend = front.replace('https://one.example/v1', 'http://127.0.0.1:4000/v1');
  const rust = back.replace('https://one.example/v1', 'http://127.0.0.1:4000/v1');
  assert.doesNotThrow(() => assertCatalogParity(frontend, rust));
});

test('detects drift of a human-readable provider label', () => {
  assert.throws(() => assertCatalogParity(front.replace("label: 'One'", "label: 'Wrong'"), back), /one.*label mismatch/i);
});
