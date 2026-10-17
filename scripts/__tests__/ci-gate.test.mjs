import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

import { SOURCES, api, decide, findDecisiveJob, latestPrRun } from "../ci/ci-gate.mjs";

const [EX63, HOSTED] = SOURCES;

function entry(source, id, status, conclusion, job) {
  return {
    source,
    run: { id, status, conclusion, html_url: `https://runs/${id}` },
    job: job === undefined ? null : { ...job, html_url: `https://jobs/${id}` },
  };
}

test("the EX63 lanes passing cancels hosted runs still working", () => {
  const verdict = decide([
    entry(EX63, 1, "completed", "success", {
      status: "completed",
      conclusion: "success",
    }),
    entry(HOSTED, 2, "in_progress", null, {
      status: "in_progress",
      conclusion: null,
    }),
  ]);
  assert.equal(verdict.state, "success");
  assert.equal(verdict.description, "CI Fast passed");
  assert.equal(verdict.targetUrl, "https://jobs/1");
  assert.deepEqual(verdict.cancel, [{ id: 2, name: "CI Fast (hosted)" }]);
});

test("hosted CI passing first also passes the gate without cancelling EX63", () => {
  const verdict = decide([
    entry(EX63, 1, "in_progress", null, {
      status: "in_progress",
      conclusion: null,
    }),
    entry(HOSTED, 3, "completed", "success", {
      status: "completed",
      conclusion: "success",
    }),
  ]);
  assert.equal(verdict.state, "success");
  assert.equal(verdict.description, "CI Fast (hosted) passed");
  assert.deepEqual(verdict.cancel, []);
});

test("a skipped decisive job is not a pass: an outsider's CI Fast run is pending on the rest", () => {
  const verdict = decide([
    entry(EX63, 1, "completed", "success", {
      status: "completed",
      conclusion: "skipped",
    }),
    entry(HOSTED, 2, "in_progress", null, {
      status: "queued",
      conclusion: null,
    }),
  ]);
  assert.equal(verdict.state, "pending");
  assert.equal(verdict.description, "Waiting on CI Fast (hosted)");
});

test("an upstream routing-only completion remains pending until fork hosted CI starts", () => {
  const verdict = decide([
    entry(EX63, 1, "completed", "success", null),
  ]);
  assert.equal(verdict.state, "pending");
  assert.match(verdict.description, /hosted CI/);
});

test("an EX63 failure waits for hosted CI instead of failing early", () => {
  const verdict = decide([
    entry(EX63, 1, "completed", "failure", {
      status: "completed",
      conclusion: "failure",
    }),
    entry(HOSTED, 3, "in_progress", null, null),
  ]);
  assert.equal(verdict.state, "pending");
});

test("everything finished with no pass fails and points at the failed job", () => {
  const verdict = decide([
    entry(EX63, 1, "completed", "failure", {
      status: "completed",
      conclusion: "failure",
    }),
    entry(HOSTED, 2, "completed", "failure", {
      status: "completed",
      conclusion: "failure",
    }),
  ]);
  assert.equal(verdict.state, "failure");
  assert.equal(verdict.description, "CI Fast: failure; CI Fast (hosted): failure");
  assert.equal(verdict.targetUrl, "https://jobs/1");
});

test("finished flows whose decisive jobs never ran fail rather than pass", () => {
  const verdict = decide([
    entry(EX63, 1, "completed", "cancelled", null),
    entry(HOSTED, 3, "completed", "cancelled", {
      status: "completed",
      conclusion: "skipped",
    }),
  ]);
  assert.equal(verdict.state, "failure");
  assert.equal(verdict.description, "No CI flow ran its checks");
});

test("no run yet is pending", () => {
  assert.equal(decide([]).state, "pending");
});

test("latestPrRun ignores push runs and picks the newest PR run", () => {
  const run = latestPrRun([
    { id: 1, event: "pull_request", created_at: "2026-09-23T10:00:00Z" },
    { id: 2, event: "push", created_at: "2026-09-23T12:00:00Z" },
    { id: 3, event: "pull_request_target", created_at: "2026-09-23T11:00:00Z" },
  ]);
  assert.equal(run.id, 3);
  assert.equal(latestPrRun([{ id: 2, event: "push", created_at: "x" }]), null);
});

test("every CI Gate source references a tracked workflow", () => {
  for (const source of SOURCES) {
    const path = fileURLToPath(
      new URL(`../../.github/workflows/${source.workflow}`, import.meta.url),
    );
    assert.ok(existsSync(path), `CI Gate references a missing workflow: ${source.workflow}`);
  }
});

test("fork hosted lane passes through the decisive job under its own caller name", () => {
  const job = findDecisiveJob(HOSTED, [
    { name: "Route (outsiders → GitHub-hosted)", conclusion: "skipped" },
    { name: "Fork PR lanes (hosted) / CI Fast (hosted) Gate", conclusion: "success" },
  ]);
  assert.equal(job?.conclusion, "success");
  assert.equal(job?.name, "Fork PR lanes (hosted) / CI Fast (hosted) Gate");
  assert.equal(
    decide([entry(HOSTED, 3, "completed", "success", job)]).state,
    "success",
  );
});

test("CI Gate does not mistake unrelated success jobs for required lanes", () => {
  assert.equal(findDecisiveJob(HOSTED, [
    { name: "Route (outsiders → GitHub-hosted)", conclusion: "success" },
    { name: "Other / CI Fast (hosted) Gate", conclusion: "success" },
  ]), null);
  assert.equal(findDecisiveJob(EX63, [
    { name: "Fork PR lanes (hosted) / CI Fast (hosted) Gate", conclusion: "success" },
  ]), null);
});

test("gate evaluation runs only on completed upstream jobs, not on every requested run", () => {
  const source = readFileSync(
    new URL("../../.github/workflows/ci-gate.yml", import.meta.url),
    "utf8",
  );
  assert.match(source, /types: \[completed\]/);
  assert.doesNotMatch(source, /types: \[requested, completed\]/);
});

test("a transient HTTP 500 when posting CI Gate status is retried", async () => {
  const original = globalThis.fetch;
  let attempts = 0;
  globalThis.fetch = async () => {
    attempts += 1;
    return {
      status: attempts === 1 ? 500 : 201,
      text: async () => JSON.stringify({ ok: true }),
    };
  };
  try {
    const result = await api("/repos/example/project/statuses/deadbeef", {
      method: "POST",
      body: { state: "success", context: "CI Gate" },
    });
    assert.equal(result.status, 201);
    assert.equal(attempts, 2);
  } finally {
    globalThis.fetch = original;
  }
});

test("CI Gate never retries permanent permission errors", async () => {
  const original = globalThis.fetch;
  let attempts = 0;
  globalThis.fetch = async () => {
    attempts += 1;
    return { status: 403, text: async () => JSON.stringify({ message: "forbidden" }) };
  };
  try {
    const result = await api("/repos/example/project/statuses/deadbeef", {
      method: "POST",
      body: { state: "success", context: "CI Gate" },
    });
    assert.equal(result.status, 403);
    assert.equal(attempts, 1);
  } finally {
    globalThis.fetch = original;
  }
});
