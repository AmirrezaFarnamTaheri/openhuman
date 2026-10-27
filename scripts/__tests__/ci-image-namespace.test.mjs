import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

test("CI image publication uses the current repository owner with lowercase GHCR tags", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/build-ci-image.yml", import.meta.url),
    "utf8",
  );
  assert.match(
    workflow,
    /REPOSITORY_OWNER: \$\{\{ github.repository_owner \}\}/,
  );
  assert.match(
    workflow,
    /name=ghcr\.io\/\$\{REPOSITORY_OWNER,,\}\/openhuman_ci/,
  );
  assert.doesNotMatch(workflow, /^\s+ghcr\.io\/tinyhumansai\/openhuman_ci:/m);
  for (const tag of ["latest", "rust-1.99.1", "sha-${{ github.sha }}"]) {
    assert.ok(workflow.includes("${{ steps.image.outputs.name }}:" + tag));
  }
});
