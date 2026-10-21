import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
);

function workflowJob(workflowPath, jobName) {
  const workflow = fs.readFileSync(path.join(repoRoot, workflowPath), "utf8");
  const jobStart = new RegExp(`^  ${jobName}:\\s*$`, "m").exec(workflow);
  assert.ok(jobStart, `${workflowPath} must define the ${jobName} job`);
  const rest = workflow.slice(jobStart.index + jobStart[0].length);
  const nextJob = /^  [A-Za-z0-9_-]+:\s*$/m.exec(rest);
  return nextJob ? rest.slice(0, nextJob.index) : rest;
}

function timeoutMinutes(workflowPath, jobName) {
  const block = workflowJob(workflowPath, jobName);
  const timeout = /^    timeout-minutes:\s*(\d+)\s*$/m.exec(block);
  assert.ok(timeout, `${workflowPath} ${jobName} must have a job timeout`);
  return Number(timeout[1]);
}

test("full Rust core tests have time to finish under hosted load", () => {
  assert.ok(
    timeoutMinutes(".github/workflows/test-reusable.yml", "rust-core-tests") >=
      120,
    "the complete Rust core suite needs at least two hours on hosted runners",
  );
});

test("independent Linux Rust integration E2E shards have time to finish the complete suite", () => {
  // The former rust-e2e-linux runner is now a reusable workflow caller; the
  // actual timeout belongs to each worker shard, not that caller.
  for (const [workflow, job] of [
    [".github/workflows/e2e-reusable.yml", "rust-e2e-linux"],
    [".github/workflows/ci-full.yml", "rust-e2e"],
  ]) {
    assert.match(
      workflowJob(workflow, job),
      /uses:\s+\.\/\.github\/workflows\/rust-e2e-shards\.yml/,
      `${workflow} must require the complete Rust E2E shard suite`,
    );
  }
  assert.ok(
    timeoutMinutes(".github/workflows/rust-e2e-shards.yml", "shard") >= 60,
    "every independent Rust integration E2E shard needs at least an hour on hosted runners",
  );
});

test("Linux full desktop E2E shards have time for the chat journey suite", () => {
  assert.ok(
    timeoutMinutes(".github/workflows/e2e-reusable.yml", "e2e-linux-full") >=
      60,
    "the full Linux desktop chat, skills, and journeys shard needs at least an hour",
  );
});
