import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const docPath = resolve(root, "docs/gitbooks/en/developing/architecture/agent-harness.md");

test("agent harness code links resolve against this checkout, not upstream main", () => {
  const markdown = readFileSync(docPath, "utf8");
  const targets = [
    "../../../../../crates/openhuman-core/src/agent/tinyagents/stop_hooks.rs",
    "../../../../../crates/openhuman-core/src/agent/registry/agents/loader.rs",
  ];
  for (const href of targets) {
    assert.ok(markdown.includes(`](${href})`), `missing relative link: ${href}`);
    assert.ok(existsSync(resolve(dirname(docPath), href)), `missing target: ${href}`);
  }
});
