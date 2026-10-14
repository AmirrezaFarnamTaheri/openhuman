import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

test("fork owner PR runs GitHub-hosted CI rather than upstream private EX63", () => {
  const temp = fs.mkdtempSync(path.join(os.tmpdir(), "openhuman-ci-route-"));
  try {
    const output = path.join(temp, "github-output.txt");
    fs.writeFileSync(output, "");
    const child = spawnSync("bash", [path.join(root, "scripts/ci/ci-fast-route.sh")], {
      cwd: root,
      encoding: "utf8",
      timeout: 1500,
      env: {
        ...process.env,
        REPO: "AmirrezaFarnamTaheri/openhuman",
        AUTHOR: "AmirrezaFarnamTaheri",
        ACTOR: "AmirrezaFarnamTaheri",
        ASSOCIATION: "OWNER",
        HEAD_SHA: "deadbeef",
        GH_TOKEN: "",
        GITHUB_OUTPUT: output,
      },
    });
    assert.equal(child.status, 0, child.stderr || String(child.error));
    const values = fs.readFileSync(output, "utf8");
    assert.match(values, /^outsider=true$/m);
    assert.match(values, /^source=fallback$/m);
  } finally {
    fs.rmSync(temp, { recursive: true, force: true });
  }
});

test("private EX63 lane is never requested by a fork repository", () => {
  const ci = fs.readFileSync(path.join(root, ".github/workflows/ci-fast.yml"), "utf8");
  assert.match(ci, /github\.repository\s*==\s*'tinyhumansai\/openhuman'/);
});
