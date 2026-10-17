import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const load = path => readFileSync(new URL("../../" + path, import.meta.url), "utf8");

test("agent-review runs through the shared E2E recipe without a second build script", () => {
  const workflow = load(".github/workflows/e2e-agent-review.yml");
  assert.match(workflow, /uses: \.\/\.github\/workflows\/e2e-reusable\.yml/);
  assert.match(workflow, /spec_path: test\/e2e\/specs\/agent-review\.spec\.ts/);
  assert.match(workflow, /run_rust_e2e: false/);
  assert.doesNotMatch(workflow, /pnpm install|cargo build|apt-get install/);
});

test("agent-review preserves its artifact retention and does not double-run Rust E2E", () => {
  const workflow = load(".github/workflows/e2e-agent-review.yml");
  const reusable = load(".github/workflows/e2e-reusable.yml");
  assert.match(workflow, /upload_artifacts_always: true/);
  assert.match(workflow, /artifact_name: e2e-agent-review-artifacts/);
  assert.match(reusable, /run_rust_e2e:\s*\n[\s\S]*?default: true/);
  assert.match(reusable, /rust-e2e-linux:\s*\n\s*if: inputs\.run_linux && inputs\.run_rust_e2e/);
  assert.match(reusable, /if: failure\(\) \|\| inputs\.upload_artifacts_always/);
});
