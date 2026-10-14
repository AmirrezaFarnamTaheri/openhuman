# Applying the source bundle to the actual OpenHuman PR

## First determine the branch state

The live PR branch must be checked against its latest head, existing changes, and current reviews before subsequent integration updates.

1. If none of the previous Prodex hardening files exist, review and apply the **complete integration patch** using `git apply --check <full.patch>` followed by `git apply <full.patch>`.
2. If the previous 11-file Prodex hardening bundle exists at the same content baseline, review and apply the **incremental peer-convergence patch** the same way. Never apply both patches.
3. If the target branch has already modified matching paths, resolve conflicts semantically rather than forcing a patch. Preserve the real host's ownership of credentials, routing, pricing and provider information.
4. Run the tests below. Then wire the new contracts into the **actual** Rust/Tauri application and run repository-native build, visual, accessibility, integration, release and deployment checks.
5. Push only after verifying all changes against latest HEAD, rerun remote CI and respond to/resolution-check each CodeRabbit review thread with links to verified code and tests.

## Local offline commands

```bash
node --test scripts/ci/tests/prodex-*.test.mjs packages/peer-fleet/test/*.test.mjs
PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/ci/tests -p 'test_*.py' -v
```

No special API keys are needed to run the offline checks. The previous Prodex gateway live smoke is **opt-in** and should use an inference-scoped virtual key, not an admin token. Do not publish OAuth tokens, private auth files, language-server extracts or full request contents in CI logs.

## Sources delivered

- `PEER_SOURCE_CENSUS.json`: complete path/hash/symbol/tag census for 951 files.
- `PEER_FILE_DECISIONS.json`: per-file machine-generated candidate/deferral/owner decision. Many entries explicitly say semantic comparison is still necessary.
- `PEER_CROSS_SOURCE_SYMBOLS.json`: lexical function/symbol and filename collisions across peers; not proof of semantic equivalence.
- `INTEGRATION_AUDIT.md`: manually curated source-to-target capability map and verification boundaries.
- `SECOND_ORDER_CONVERGENCE.md`: cross-module ownership design, high-level components and acceptance gates.
- `packages/peer-fleet/`: executable first/second-order integration contracts and tests, intentionally isolated from the unavailable target source.
