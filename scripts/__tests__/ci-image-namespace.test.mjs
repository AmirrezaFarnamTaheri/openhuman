import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
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


test("production GHCR image tags, verification, and latest promotion normalize mixed-case fork owners", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/release-production.yml", import.meta.url),
    "utf8",
  );
  const build = workflow.split("\n  build-docker:\n")[1]?.split("\n  build-cli-linux:\n")[0];
  const promote = workflow.split("\n  tag-docker-latest:\n")[1]?.split("\n  record-sentry-deploy:\n")[0];
  assert.ok(build, "production build-docker job exists");
  assert.ok(promote, "production tag-docker-latest job exists");

  const expressions = [
    'base="${REGISTRY}/${IMAGE_NAME,,}"',
    'image="${REGISTRY}/${IMAGE_NAME,,}:${TAG}"',
    'src="${REGISTRY}/${IMAGE_NAME,,}:${TAG}"',
    'dst="${REGISTRY}/${IMAGE_NAME,,}:latest"',
  ];
  assert.ok(build.includes(expressions[0]), "versioned GHCR tags must lowercase the namespace");
  assert.ok(build.includes(expressions[1]), "pullability verification must lowercase the namespace");
  assert.ok(promote.includes(expressions[2]), "promotion source must lowercase the namespace");
  assert.ok(promote.includes(expressions[3]), "promotion destination must lowercase the namespace");
  assert.ok(
    build.includes("      - name: Verify pushed image is pullable\n        env:\n          TAG: ${{ needs.prepare-build.outputs.tag }}"),
    "verification must use the validated production tag",
  );

  const output = execFileSync("bash", ["-c", [
    "set -euo pipefail",
    "REGISTRY=ghcr.io",
    "IMAGE_NAME=AmirrezaFarnamTaheri/openhuman-core",
    "TAG=v0.64.23",
    ...expressions,
    'echo "$base:$TAG"',
    'echo "$image"',
    'echo "$src"',
    'echo "$dst"',
  ].join("\n")], { encoding: "utf8" }).trim().split("\n");
  assert.deepEqual(output, [
    "ghcr.io/amirrezafarnamtaheri/openhuman-core:v0.64.23",
    "ghcr.io/amirrezafarnamtaheri/openhuman-core:v0.64.23",
    "ghcr.io/amirrezafarnamtaheri/openhuman-core:v0.64.23",
    "ghcr.io/amirrezafarnamtaheri/openhuman-core:latest",
  ]);
});
