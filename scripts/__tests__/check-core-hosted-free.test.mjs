import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import {
  CORE_SRC,
  checkRepository,
  formatReport,
  scanSource,
  stripRustComments,
} from "../ci/check-core-hosted-free.mjs";

const CHECKER = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../ci/check-core-hosted-free.mjs",
);

function fixture(files) {
  const root = mkdtempSync(join(tmpdir(), "core-hosted-free-"));
  for (const [path, body] of Object.entries(files)) {
    const full = join(root, CORE_SRC, path);
    mkdirSync(dirname(full), { recursive: true });
    writeFileSync(full, body);
  }
  return root;
}

test("comments are ignored but string literals are scanned", () => {
  const src = [
    "// crate::backend in a line comment",
    '/// "https://api.tinyhumans.ai" in a doc comment',
    "/* /teams/me /* nested */ still a comment */",
    "let ok = 1;",
    'let url = "https://api.tinyhumans.ai/x";',
    'let raw = r#"/voice-agent/start"#;',
    'let quote = \'"\'; let route = "/openai/v1/chat";',
  ].join("\n");
  const hits = scanSource("x.rs", src);
  assert.deepEqual(
    hits.map((h) => [h.line, h.name]),
    [
      [5, "tinyhumans.ai"],
      [6, "/voice-agent"],
      [7, "/openai/v1"],
    ],
  );
});

test("every forbidden pattern is detected in code", () => {
  const code = [
    "use crate::backend::BackendClient;",
    'const A: &str = "agent-integrations/x";',
    'const B: &str = "/telemetry/langfuse/ingestion";',
    'const C: &str = "/memory/free-period";',
    'const D: &str = "/teams/me/usage";',
  ].join("\n");
  assert.equal(scanSource("x.rs", code).length, 5);
});

test("complete third-party provider URLs may use the OpenAI-compatible path", () => {
  for (const literal of [
    '"https://api.synthetic.new/openai/v1"',
    'r#"https://provider.example/openai/v1/chat/completions"#',
    'b"http://localhost:8000/openai/v1"',
  ]) {
    assert.deepEqual(
      scanSource("providers.rs", `const URL = ${literal};`),
      [],
      literal,
    );
  }
});

test("a provider URL cannot hide a separate hosted route on the same line", () => {
  const code =
    'let provider = "https://api.synthetic.new/openai/v1"; let route = "/openai/v1/chat";';
  assert.deepEqual(
    scanSource("x.rs", code).map((hit) => hit.name),
    ["/openai/v1"],
  );
});

test("relative, composed and hosted OpenAI paths still belong outside the core", () => {
  for (const code of [
    'let route = "/openai/v1/chat";',
    'format!("{}/openai/v1", backend);',
    'format!("https://{host}/openai/v1");',
    'concat!("https://api.example", "/openai/v1");',
    'let url = "https://api.tinyhumans.ai/openai/v1";',
    'let url = r#"https://tinyhumans.ai/openai/v1"#;',
    'let json = r#"{"url":"https://api.example/openai/v1"}"#;',
  ]) {
    assert.ok(
      scanSource("x.rs", code).some((hit) => hit.name === "/openai/v1"),
      code,
    );
  }
});

test("stripping keeps line numbers", () => {
  const src = "a\n/* one\ntwo */\nb\n";
  assert.equal(
    stripRustComments(src).split("\n").length,
    src.split("\n").length,
  );
});

test("_tests.rs files are skipped and a clean tree passes", () => {
  const root = fixture({
    "lib.rs": "pub fn f() {}\n",
    "thing/mod.rs": "// crate::backend was removed\n",
    "thing/mod_tests.rs": 'const X: &str = "https://api.tinyhumans.ai";\n',
  });
  try {
    assert.deepEqual(checkRepository(root), []);
    const run = spawnSync("node", [CHECKER, root], { encoding: "utf8" });
    assert.equal(run.status, 0, run.stdout + run.stderr);
    assert.match(run.stdout, /ok/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("a hosted reference fails the checker with file and line", () => {
  const root = fixture({
    "a/mod.rs": 'fn x() {}\nconst U: &str = "https://api.tinyhumans.ai";\n',
  });
  try {
    const hits = checkRepository(root);
    assert.equal(hits.length, 1);
    assert.match(formatReport(hits), /a\/mod\.rs:2: tinyhumans\.ai/);
    const run = spawnSync("node", [CHECKER, root], { encoding: "utf8" });
    assert.equal(run.status, 1);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});

test("a missing core tree exits 2 instead of passing vacuously", () => {
  const root = mkdtempSync(join(tmpdir(), "core-hosted-free-empty-"));
  try {
    const run = spawnSync("node", [CHECKER, root], { encoding: "utf8" });
    assert.equal(run.status, 2);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
