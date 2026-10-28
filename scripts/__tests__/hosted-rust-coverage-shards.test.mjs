import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  AREA_ENV,
  buildPlan,
  hostedMatrix,
  validatePlan,
} from "../ci/self-hosted/lanes-plan.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const areas = Object.fromEntries(Object.keys(AREA_ENV).map((key) => [key, true]));
const shards = ["core", "facades", "integrations"];

test("hosted coverage shards partition the full Rust suite without losing checks", () => {
  const hosted = buildPlan({ profile: "hosted", areas });
  assert.deepEqual(validatePlan(hosted), []);
  const groups = hostedMatrix(hosted).map((group) => group.group);
  assert.deepEqual(
    groups.filter((group) => group.startsWith("rust-cov")),
    shards.map((shard) => `rust-cov-${shard}`),
  );
  for (const shard of shards) {
    const lane = hosted.lanes.find((entry) => entry.name === `rust-cov-${shard}`);
    assert.ok(lane?.active, `missing active ${shard} lane`);
    const check = lane.checks.find((item) => item.name === "rust-core-coverage");
    assert.ok(check?.when);
    assert.equal(check.env.OH_COV_SHARD, shard);
    assert.equal(check.env.OUT, `ci-out/lcov/lcov-core-${shard}.info`);
    assert.equal(check.env.OH_COV_RUNNER, shard === "core" ? "nextest" : undefined);
    assert.ok(
      lane.checks.some((item) => item.name === "test-modules"),
      "fixtures must be installed in every runner",
    );
  }
  assert.equal(hosted.lanes.some((entry) => entry.name === "rust-cov"), false);
});

test("private EX63 retains its original whole-suite coverage behavior", () => {
  const privatePlan = buildPlan({
    profile: "ex63",
    areas,
    env: { CI_SCRATCH_DIR: "/scratch" },
  });
  assert.deepEqual(validatePlan(privatePlan), []);
  const lane = privatePlan.lanes.find((entry) => entry.name === "rust-cov");
  assert.ok(lane?.active);
  const check = lane.checks.find((item) => item.name === "rust-core-coverage");
  assert.equal(check.env.OH_COV_SHARD, undefined);
  assert.equal(check.env.OH_COV_RUNNER, "nextest");
});

test("hosted gate demands three reports and preserves whole-tree plus diff coverage", () => {
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/ci-lanes.yml"), "utf8");
  for (const shard of shards) {
    assert.ok(workflow.includes(`--wait rust-cov-${shard}`));
  }
  assert.ok(workflow.includes('lcov-core-*.info'));
  assert.ok(workflow.includes('"${#reports[@]}" -ne 3'));
  assert.ok(workflow.includes('assert-coverage-presence.sh merged-rust-core.info --all'));
  assert.ok(workflow.includes('diff-cover.sh ci-artifacts'));
  assert.match(workflow, /cargo-nextest/);
});

test("facade shard serializes singleton embed tests without skipping coverage", () => {
  const coverage = fs.readFileSync(path.join(root, "scripts/ci/rust-coverage.sh"), "utf8");
  assert.match(
    coverage,
    /suite "openhuman-embed" llvm_cov_embed --no-report --no-fail-fast -p openhuman-embed --all-targets -- \\\n\s+--test-threads=1/,
  );
  assert.match(coverage, /suite "openhuman-rpc" llvm_cov_package/);
  assert.match(coverage, /suite "openhuman-tinyhumans" llvm_cov_product_facade/);
});

test("all three Rust shards retain the original suite and one aggregate presence gate", () => {
  const coverage = fs.readFileSync(path.join(root, "scripts/ci/rust-coverage.sh"), "utf8");
  assert.match(coverage, /OH_COV_SHARD:-full/);
  for (const shard of shards) {
    assert.match(coverage, new RegExp(`has_cov_section ${shard}`));
  }
  assert.match(coverage, /COV_SHARD.*full/);
  assert.ok(coverage.includes('assert-coverage-presence.sh'));
  assert.match(coverage, /whole-tree coverage presence deferred/);
});
