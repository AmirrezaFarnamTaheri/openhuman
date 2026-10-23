import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const source = (path) =>
  readFileSync(new URL(`../../app/src/${path}`, import.meta.url), "utf8");

test("Connections and Memory keep nested views within the supported desktop route chunks", () => {
  for (const page of ["pages/Skills.tsx", "pages/Memory.tsx"]) {
    assert.doesNotMatch(
      source(page),
      /\bimport\s*\(/,
      `${page} must use static component imports`,
    );
  }
});

function checkRouteLoading(routes) {
  if (/\blazy\s*\(/.test(routes)) {
    const imports = [...routes.matchAll(/\bimport\s*\(([^)]*)\)/g)];
    assert.ok(imports.length > 0);
    for (const [call, path] of imports) {
      assert.match(path, /^['"]\.\/pages\/[\w/-]+['"]$/);
      assert.ok(routes.includes(`lazy(() => ${call})`));
    }
    assert.match(routes, /lazy\(\(\) => import\(['"]\.\/pages\/[\w/-]+['"]\)\)/);
    assert.match(routes, /<Suspense\s+fallback=/);
  } else {
    assert.match(routes, /import Skills from ['"]\.\/pages\/Skills['"]/);
    assert.doesNotMatch(routes, /\bimport\s*\(/);
  }
}

test("top-level routes use static imports or literal lazy imports with a loading boundary", () => {
  checkRouteLoading(source("AppRoutes.tsx"));
});

test("a split route without a loading boundary is rejected", () => {
  const split = "const Skills = lazy(() => import('./pages/Skills'));";
  assert.throws(() => checkRouteLoading(split));
  assert.doesNotThrow(() => checkRouteLoading(split + '<Suspense fallback={<Loading />}>'));
  assert.throws(() => checkRouteLoading("const Skills = lazy(() => import(path));<Suspense fallback={null}>"));
  assert.throws(() => checkRouteLoading(split + "const Other = lazy(() => import(path));<Suspense fallback={null}>"));
});
