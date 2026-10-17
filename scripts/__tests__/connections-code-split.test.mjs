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

test("top-level route splitting retains a loading boundary", () => {
  const routes = source("AppRoutes.tsx");
  assert.match(routes, /lazy\(\(\) => import\('\.\/pages\/Skills'\)\)/);
  assert.match(routes, /<Suspense\s+fallback=/);
});
