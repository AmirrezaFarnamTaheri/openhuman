import assert from "node:assert/strict";
import test from "node:test";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
  canonicalMergeBase,
  validateCoverageDebt,
} from "../ci/storage-bypass-debt.mjs";

test("fork debt uses integrated canonical history and still refuses fork writers", () => {
  const root = mkdtempSync(join(tmpdir(), "storage-canonical-history-"));
  const git = (args) =>
    execFileSync("git", args, {
      cwd: root,
      encoding: "utf8",
      stdio: ["ignore", "pipe", "pipe"],
    }).trim();
  const commit = (message) => {
    git(["add", "history"]);
    git([
      "-c",
      "user.name=Fixture",
      "-c",
      "user.email=fixture@example.invalid",
      "commit",
      "-m",
      message,
    ]);
    return git(["rev-parse", "HEAD"]);
  };
  try {
    git(["init"]);
    writeFileSync(join(root, "history"), "fork base\n");
    const forkBase = commit("fork base");
    git(["update-ref", "refs/remotes/origin/main", forkBase]);
    writeFileSync(join(root, "history"), "integrated canonical writer\n");
    const integrated = commit("canonical update");
    git(["branch", "canonical-next"]);
    writeFileSync(join(root, "history"), "fork-only writer\n");
    commit("fork work");
    git(["checkout", "canonical-next"]);
    writeFileSync(join(root, "history"), "later upstream change\n");
    const later = commit("not yet integrated");
    git(["checkout", "-"]);

    const calls = [];
    const base = canonicalMergeBase(
      (args) => {
        calls.push(args);
        if (args[0] === "fetch") {
          assert.deepEqual(args, [
            "fetch",
            "--no-tags",
            "--no-recurse-submodules",
            "https://github.com/tinyhumansai/openhuman.git",
            "refs/heads/main:refs/remotes/openhuman-canonical/main",
          ]);
          // Network-free fixture: install the separately advanced canonical ref.
          return git([
            "update-ref",
            "refs/remotes/openhuman-canonical/main",
            later,
          ]);
        }
        return git(args);
      },
      {
        GITHUB_REPOSITORY: "contributor/openhuman",
        GITHUB_BASE_REF: "custom-head",
      },
    );
    assert.equal(base, integrated);
    assert.notEqual(base, forkBase);
    assert.notEqual(base, later);
    assert.equal(calls.filter((args) => args[0] === "fetch").length, 1);

    const inherited = {
      rule: "json-write",
      path: "crates/openhuman-embed/src/canonical.rs",
      text: "std::fs::write(path, bytes);",
      occurrence: 1,
      classification: "external-export",
      reason: "Existing canonical artifact export at this exact callsite.",
      issue: "https://github.com/tinyhumansai/openhuman/issues/7428",
    };
    const added = { ...inherited, path: "crates/openhuman-embed/src/fork.rs" };
    const history = { baseline: [], extension: null, findings: [inherited] };
    assert.deepEqual(
      validateCoverageDebt([inherited], [], [inherited], history),
      [],
    );
    assert.match(
      validateCoverageDebt(
        [inherited, added],
        [],
        [inherited, added],
        history,
      ).join("\n"),
      /branch-added writer/,
    );
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("canonical PRs keep their base without a network fetch", () => {
  const calls = [];
  assert.equal(
    canonicalMergeBase(
      (args) => {
        calls.push(args);
        return "base\n";
      },
      {
        GITHUB_REPOSITORY: "tinyhumansai/openhuman",
        GITHUB_BASE_REF: "release",
      },
    ),
    "base",
  );
  assert.deepEqual(calls.at(-1), ["merge-base", "origin/release", "HEAD"]);
  assert.equal(
    calls.some((args) => args[0] === "fetch"),
    false,
  );
});

test("a failed canonical fetch cannot fall back to a permissive fork baseline", () => {
  const calls = [];
  assert.throws(
    () =>
      canonicalMergeBase(
        (args) => {
          calls.push(args);
          if (args[0] === "fetch") throw new Error("canonical unavailable");
          return "head";
        },
        {
          GITHUB_REPOSITORY: "contributor/openhuman",
          GITHUB_BASE_REF: "release",
        },
      ),
    /canonical unavailable/,
  );
  assert.equal(
    calls.some((args) => args[0] === "merge-base"),
    false,
  );
  assert.equal(
    calls.at(-1).at(-1),
    "refs/heads/release:refs/remotes/openhuman-canonical/release",
  );
});
