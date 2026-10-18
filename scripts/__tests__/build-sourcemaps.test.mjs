import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

import { shouldGenerateSourcemaps } from "../../app/scripts/build-sourcemaps.mjs";

test("ordinary and CI builds omit large source-map artifacts", () => {
  assert.equal(shouldGenerateSourcemaps({}), false);
  assert.equal(shouldGenerateSourcemaps({ SENTRY_AUTH_TOKEN: "  " }), false);
  assert.equal(shouldGenerateSourcemaps({ OPENHUMAN_BUILD_SOURCEMAPS: "0" }), false);
});

test("Sentry upload and explicitly requested diagnostic builds retain source maps", () => {
  assert.equal(shouldGenerateSourcemaps({ SENTRY_AUTH_TOKEN: "real-token" }), true);
  assert.equal(shouldGenerateSourcemaps({ OPENHUMAN_BUILD_SOURCEMAPS: "1" }), true);
});

test("Vite uses the policy instead of generating source maps unconditionally", () => {
  const config = readFileSync(new URL("../../app/vite.config.ts", import.meta.url), "utf8");
  assert.match(config, /sourcemap:\s*shouldGenerateSourcemaps\(\)/);
  assert.doesNotMatch(config, /sourcemap:\s*true,/);
});
