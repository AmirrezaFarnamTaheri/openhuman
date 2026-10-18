# Science and agent-platform peer audit

Inspected 2026-10-18 through the GitHub connector, with one primary-source license compatibility lookup. This is a static source audit. No peer code was installed, executed, imported into OpenHuman, or ported. Test files and workflows below were read; their presence is not a claim that their current CI passes. Effort estimates are engineering estimates for a focused implementation, tests and documentation, not commitments.

## Recommendation

Prioritize **observable provenance with explicit uncertainty** and **resource-aware execution planning**. Both add information OpenHuman can act on without replacing its tool registry, harness or permission model. Treat general agent frameworks, large persona catalogs and generic finish reminders as lower-value adoption targets because OpenHuman already owns those responsibilities. Retain existing owner boundaries: policy and product composition in OpenHuman; generic lifecycle observation in TinyAgents; bounded native file operations in TinyTools; skill discovery/ranking/install mechanics in TinySkills.

A future science pack should be opt-in and small. A useful first scope is a versioned research record, local dataset profile, exact command/result provenance and a reviewer export. It should use actual installed tools and preserve the distinction between a supplied claim, an observed result, a content hash and a verified conclusion. None of the peer examples supplies a reason to widen the current PR's agent schema or add new agents.

## Source snapshots

Every source link in this report is pinned to the inspected commit. The date is the main-branch commit's committer date, not a guarantee that all code was written that day.

| Repository | Exact main commit | Commit date | Inspection depth |
| --- | --- | --- | --- |
| K-Dense-AI/science-superpowers | `0374bdf82c2372bdf1667e3c762146c3a29215f5` | 2026-09-13 | Preregistration implementation, test script, verification skill, complete tree, license |
| K-Dense-AI/claude-scientific-writer | `529b9f73ab4b48925027145800d7b49e96ccfd3c` | 2026-09-29 | Core stop hook, artifact models/scanner, metadata parsing tests, CI, license and nested document license |
| K-Dense-AI/agentic-data-scientist | `702dee9dfb9e5165ce51b52ad7b7885948df7cd1` | 2026-08-18 | Review confirmation, loop detector, stage terminal status, tree/tests, license |
| K-Dense-AI/claude-skills-mcp | `72a7b502237cb3e697d3944be1b778498eed2d87` | 2026-07-20 | Search engine and cache implementation, test inventory, license |
| K-Dense-AI/scientific-agent-skills | `a1043ef544e3a100c28e70a736b6cf32554d0677` | 2026-10-12 | Resource planner, resource regressions, bounded dataset audit entry point, skill CI, tree, license |
| K-Dense-AI/k-dense-byok | `6a55f5ce6f4bd9ac7ff75d01ec20d7f27073a9f0` | 2026-10-16 | Provenance recorder, reviewer-export renderer/static verifier, related test inventory, license |
| K-Dense-AI/mimeo | `a4cea1842a24515cda354a3aa9eb9763677a3cdd` | 2026-09-02 | Untrusted-block formatting, test/module inventory, license |
| K-Dense-AI/scientific-agents | `98c7fae46648724e1a43a1f2b207b0c82fcc462e` | 2026-10-02 | Catalog/package layout, validation workflow presence, license; individual personas not audited |
| multica-ai/multica | `9b43430438e3a2dce63906127d337663cc488c59` | 2026-10-18 | Skill bundle hash, archive importer, related tests, complete tree, full additional license conditions |
| multica-ai/dsh-multica-runtime | `e29aae228449dfe50e88af60ef4281e38ca44e2a` | 2026-08-14 | Task environment filter, package manifest, complete tree |
| multica-ai/andrej-karpathy-skills | `2c606141936f1eeef17fa3043a72095b4765b9c2` | 2026-04-20 | Complete tree; prompt package only, no implementation reuse assessed |

## Candidate inventory, from small details to complete modules

### 1. Record observed provenance separately from inferred edges

**Priority: high. Owner: TinyAgents observation interfaces plus OpenHuman artifact persistence. Effort: medium to large, approximately 4–8 engineering days for a narrow first version.**

K-Dense BYOK records tool starts/ends and builds file edges from declared paths or before/after sandbox snapshots. Its asynchronous scan queue explicitly downgrades attribution when queued work means the scan could include another call's writes. This is a stronger model than attributing every changed file to the most recent tool. The same file documents a baseline race that can miss a first write; observation is intentionally incomplete. [Recorder source](https://github.com/K-Dense-AI/k-dense-byok/blob/6a55f5ce6f4bd9ac7ff75d01ec20d7f27073a9f0/server/src/provenance/recorder.ts)

**OpenHuman comparison:** the recovered `coding_handoff` context packet intentionally contains untrusted reports and grants no evidence authority. The TinyAgents finish-policy work distinguishes actual child tool calls from envelopes. An observational artifact layer could bind those real call IDs to workspace-relative revisions, then add `observed`, `inferred`, `missing` and `stale` statuses. It should not promote a handoff's supplied exit code to runtime verification.

**Smallest implementation:** observe known structural write receipts and existing process results first. Those already provide exact target revisions or actual completion states. Add opaque shell snapshot attribution only as an explicitly partial later feature. A bounded queue, per-run identifiers, cancellation handling and startup-baseline completion must be specified; the peer's unbounded queue and hard-coded tool-name lists should not be copied into the generic vocabulary.

### 2. Export a reviewer package without implying execution or authenticity

**Priority: high after provenance exists. Owner: OpenHuman export/artifact subsystem. Effort: medium, 3–5 days.**

The BYOK export code distinguishes matched historical bytes from current unverified copies, emits a missing-information record and provides a static hash verifier. Packaging does not replay commands. Its Markdown renderer selects a fence longer than any embedded backtick run, a useful small detail for safely presenting captured snippets. The verifier checks relative paths, duplicate entries, symlinks, unexpected files and a byte ceiling. [Export renderer and verifier](https://github.com/K-Dense-AI/k-dense-byok/blob/6a55f5ce6f4bd9ac7ff75d01ec20d7f27073a9f0/server/src/evidence/report.ts)

**OpenHuman comparison:** the existing artifact offload uses scoped blob storage and materializes recoverable tool output. That provides storage but not a portable, source-linked review bundle. A future export should reuse that storage and add an inventory, content revisions, selected raw results, explicit omissions and current-versus-captured identity. It must preserve existing tenant scope and scrubbing rules. A checksum inventory alone is not a signature, proof of a successful rerun, or a license to redistribute data.

**Useful bounded increment:** export selected receipts and artifacts as data with an independent integrity check. Do not embed a runnable recovery script, install dependencies or claim broad RO-Crate provenance conformance merely because a JSON-LD descriptor exists.

### 3. Plan concurrency from effective resources, including zero feasible workers

**Priority: high. Owner: host execution planning; a pure arithmetic helper can be separate from machine probes. Effort: medium, 2–4 days.**

The scientific-agent-skills planner combines effective CPU capacity, memory after reserve, task count and the requested worker ceiling. It can return zero when no worker fits memory. Unknown memory becomes a review-required/conservative plan, while accelerator discovery remains a candidate count with usability unverified. Tests cover cgroup ancestors, scheduler allocations, unknown memory, zero-worker plans and nonexecuting accelerator decisions. [Planner](https://github.com/K-Dense-AI/scientific-agent-skills/blob/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/get-available-resources/scripts/plan_workload.py), [behavioral tests](https://github.com/K-Dense-AI/scientific-agent-skills/blob/a1043ef544e3a100c28e70a736b6cf32554d0677/tests/get-available-resources/test_scripts.py)

**OpenHuman comparison:** the present development session has required manual serialization of Rust compilation and large Node heaps. The product's parallel agent limits should similarly be distinguished from the machine's executable workload capacity. This is not a reason for a model to inspect arbitrary process environments or for a tool to change host scheduling policy.

**Smallest implementation:** a typed host resource snapshot and deterministic recommendation with explicit unknown fields and binding limits. Start with CPU/memory and keep probes behind existing OS/platform abstractions. Do not silently convert an unknown capacity or an insufficient-memory result into permission to start one job. Recommendations are estimates; they are not reservations.

### 4. Add a bounded local dataset profile as an optional science capability

**Priority: medium. Owner: optional native analysis tool plus existing host filesystem gate. Effort: medium, 3–6 days for CSV/TSV only.**

The missingness/leakage CLI accepts bounded local CSV/TSV, row and file limits, named group/entity/split/time columns and opt-in identifier disclosure. Its output labels overlap as a possible concern rather than proof of leakage, and absence of overlap as insufficient evidence of independence. The entry point delegates analysis and file checks to distinct modules, so its CLI flags are not themselves evidence that every lower-level boundary is correct. [Audited entry point](https://github.com/K-Dense-AI/scientific-agent-skills/blob/a1043ef544e3a100c28e70a736b6cf32554d0677/skills/exploratory-data-analysis/scripts/missingness_leakage_audit.py)

**OpenHuman comparison:** structural source tools already establish a useful contract: bounded reads, content revisions, paginated JSON and gate-controlled paths. A dataset inspector can follow that design without inserting statistics or scientific dependencies into `tinytools` itself. Start with counts, missingness and explicit split overlap; defer automatic causal conclusions, model quality labels and clinical interpretation.

### 5. Preserve preregistration integrity and distinguish chronology from a public timestamp

**Priority: medium, science pack only. Owner: OpenHuman workflow record or separate pure audit helper. Effort: medium, 2–4 days for a read-only local audit.**

`prereg.sh` records a first Git commit and data blob hashes, then audits registration contents at HEAD/worktree and result-commit ancestry. Its five repository-based regressions cover placeholder/no-placeholder stamping, edited registrations, early outputs and changed raw data. [Audit script](https://github.com/K-Dense-AI/science-superpowers/blob/0374bdf82c2372bdf1667e3c762146c3a29215f5/skills/preregistering-analysis/prereg.sh), [tests](https://github.com/K-Dense-AI/science-superpowers/blob/0374bdf82c2372bdf1667e3c762146c3a29215f5/scripts/test-prereg.sh)

**Adoption constraint:** the script's `freeze` mutates files and creates two Git commits. It is not suitable for a supposedly read-only tool or unconditional onboarding hook. It also documents unsupported whitespace paths. Parallel-branch and uncommitted-output warnings can coexist with an overall zero exit code, and no committed outputs are not proof that a preregistered analysis has run. The first useful port is a read-only structured audit with separate `integrity`, `chronology`, `data_match` and `unverified` fields, not a single success boolean. Local Git ancestry supports local ordering; it is not independent public preregistration or trusted wall-clock evidence.

### 6. Keep verification guidance domain-specific, but let the host decide completion

**Priority: low as code; medium as a narrowly selected skill. Owner: OpenHuman/TinySkills prompt composition. Effort: small, 0.5–1 day with prompt-budget tests.**

The verification skill requires current output, assumptions, effect sizes/intervals and actual artifact inspection before a research claim. The writer's stop hook supplies one bounded completion reminder and refuses to recurse when already inside a stop hook. [Science verification guidance](https://github.com/K-Dense-AI/science-superpowers/blob/0374bdf82c2372bdf1667e3c762146c3a29215f5/skills/verifying-results-before-claiming/SKILL.md), [bounded stop hook](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/scientific_writer/core.py)

**OpenHuman comparison:** this PR already restores a finish policy tied to real calls and current-revision guidance. Installing another generic stop loop would overlap with that and could add extra turns without new evidence. Select research-specific explanation through a real skill only when requested; keep runtime verification in the existing harness.

### 7. Clear stale review decisions and persist honest terminal status

**Priority: medium as regression ideas. Owner: TinyAgents lifecycle state and OpenHuman run presentation. Effort: small to medium, 1–2 days.**

Agentic Data Scientist clears a previous review decision before invoking the reviewer and uses distinct state keys for plan and implementation review. Its terminal status requires a nonempty criteria list, preserves unmet criteria/unapproved attempted stages and emits persistence deltas. These are useful examples of avoiding an empty-list success or a stale prior decision. [Review confirmation](https://github.com/K-Dense-AI/agentic-data-scientist/blob/702dee9dfb9e5165ce51b52ad7b7885948df7cd1/src/agentic_data_scientist/agents/adk/review_confirmation.py), [stage orchestration](https://github.com/K-Dense-AI/agentic-data-scientist/blob/702dee9dfb9e5165ce51b52ad7b7885948df7cd1/src/agentic_data_scientist/agents/adk/stage_orchestrator.py)

**Limit:** the decision remains model output; the callback reads a dictionary's truthiness and relies on ADK private invocation/event fields. Do not import this controller. Use generation-scoped typed states and actual task IDs in existing runners, and test reuse, cancellation and missing decisions. A completed-with-warnings label should be driven by the host's evidence model, not by a model-written approval bit.

### 8. Scope loop detection to the emitting agent and one invocation

**Priority: medium if existing loop telemetry demonstrates a gap. Owner: TinyAgents. Effort: medium, 2–3 days.**

The peer detector resets buffers per invocation, restricts observation to its own agent, and stops that agent without ending the whole workflow. It uses a bounded sliding character window with conservative repeated-pattern thresholds. [Loop detector](https://github.com/K-Dense-AI/agentic-data-scientist/blob/702dee9dfb9e5165ce51b52ad7b7885948df7cd1/src/agentic_data_scientist/agents/adk/loop_detection.py)

**Adoption constraint:** its private ADK state-saving bridge, broad exception handling and text heuristics are framework-specific. Before implementing anything, compare existing TinyAgents repetition handling on actual traces. A regression proving that repeated child output cannot trip the parent's detector is more useful than another generic detector. Legitimate repetitive code/data must remain possible.

### 9. Cache skill embeddings by content and model, only if lexical search needs improvement

**Priority: low until relevance is measured. Owner: TinySkills ranking implementation, host-visible corpus unchanged. Effort: medium, 3–5 days plus evaluation corpus.**

The MCP search engine embeds name plus description, lazily loads its model, and caches batches by model name plus exact texts. It verifies the row count of a cached matrix and falls back to re-encoding on load failure. [Search/cache implementation](https://github.com/K-Dense-AI/claude-skills-mcp/blob/72a7b502237cb3e697d3944be1b778498eed2d87/packages/backend/src/claude_skills_mcp_backend/search_engine.py)

**OpenHuman comparison:** `skills/search.rs` already returns a capped projection and delegates ranking to TinySkills while the host selects trusted roots. Keep that split. A vector backend would add model download, disk/RAM and cache-lifecycle costs; the peer's cache only checks one shape dimension and has no eviction budget in this file. Candidate improvements include version/model/content cache identity, validated finite/dimensioned arrays and lexical fallback, not replacing discovery with a new Python service.

### 10. Use document-neutral artifact inventories and narrowly tested metadata parsing

**Priority: medium for artifact UI; low for parser portability. Owner: OpenHuman artifact presentation or a document-specific module. Effort: small to medium, 1–3 days.**

The writer preserves legacy PDF/TeX fields while adding generic final/draft/source/artifact lists and document-neutral aliases. Its tests cover nested-brace LaTeX titles, formatted text in word estimates and excluding BibTeX directives from citation counts. [Models](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/scientific_writer/models.py), [scanner/parsers](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/scientific_writer/utils.py), [tests](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/tests/test_utils.py)

**Adoption constraint:** the peer scanner walks the entire project recursively and the parsers read whole files. OpenHuman should use its existing artifact store, bounded enumeration and host gate. Citation counts are metadata; they do not establish that references exist or support the prose. Regex word counting should remain an explicitly approximate display, not a publication compliance certificate.

### 11. Make content identities unambiguous and independent of file enumeration order

**Priority: medium design reference; direct reuse blocked by license assessment. Owner: TinySkills bundle identity. Effort: small, 0.5–1 day for an independently specified contract and tests.**

Multica sorts files by path, includes per-file hashes and sizes, and hashes length-prefixed fields including a version string and source identity. Two tests cover order independence and changed primary content. [Bundle hash](https://github.com/multica-ai/multica/blob/9b43430438e3a2dce63906127d337663cc488c59/server/pkg/skillbundle/hash.go), [tests](https://github.com/multica-ai/multica/blob/9b43430438e3a2dce63906127d337663cc488c59/server/pkg/skillbundle/hash_test.go)

**OpenHuman comparison:** skill install acknowledgements already bind to a document digest. Structural chunks in the current adapter bind both manifest and template content. A multi-file skill format should define canonical paths, duplicate handling, field boundaries and versioning once in its owner. Do not copy Multica's implementation or notices into a GPL component on the assumption that its license is ordinary Apache.

### 12. Treat archive path normalization and collisions as one contract

**Priority: medium regression inventory; direct reuse blocked by license assessment. Owner: TinySkills installer. Effort: small to medium, 1–2 days if archive import is actually in scope.**

Multica handles Windows backslash entries, directory markers, absolute/drive/NUL paths and collisions after normalization. It accepts a root or single-wrapper skill layout and prevents nested `SKILL.md` files from becoming supporting assets. The importer skips some individually invalid/oversized assets but aborts aggregate cap failures. [Archive importer](https://github.com/multica-ai/multica/blob/9b43430438e3a2dce63906127d337663cc488c59/server/internal/handler/skill_import_archive.go)

**OpenHuman comparison:** the current URL installer fetches a bounded Markdown document through TinySkills scanning and digest acknowledgement. An archive format would be a separate new capability. Reuse the test questions, including platform-independent path interpretation and duplicate canonical names; define whether partial imports are allowed and make omissions visible. Do not add ZIP support merely to match another product.

### 13. Delimit source text without granting it instruction authority

**Priority: small hardening review. Owner: existing prompt rendering. Effort: small, 0.5–1 day.**

Mimeo sanitizes metadata into a single line, validates wrapper tag names, and breaks matching opening/closing tag sequences inside source text. [Prompt-boundary helper](https://github.com/K-Dense-AI/mimeo/blob/a4cea1842a24515cda354a3aa9eb9763677a3cdd/src/mimeo/prompt_safety.py)

**OpenHuman comparison:** coding handoffs are structured bounded data preceded by an untrusted-report notice. A malicious reference must not gain authority by closing a visual delimiter. Use adversarial rendering regressions where a real renderer needs them. Delimiters do not constitute a security boundary or guarantee that a model ignores embedded instructions. Keep JSON validation, permissions and trusted tool-result evidence separate.

### 14. Credential forwarding should be narrowly scoped, but the inspected bridge is not reusable

**Priority: reference only. Owner: OpenHuman runtime host. Effort: small for a regression; no port proposed.**

The DSH bridge builds a fresh environment map containing only a particular task-token class rather than copying a complete parent environment. This illustrates the narrow-interface principle, but its package is explicitly `private` and `UNLICENSED`, with prerelease DeepSeek harness peers. [Task environment](https://github.com/multica-ai/dsh-multica-runtime/blob/e29aae228449dfe50e88af60ef4281e38ca44e2a/src/task-env.ts), [package manifest](https://github.com/multica-ai/dsh-multica-runtime/blob/e29aae228449dfe50e88af60ef4281e38ca44e2a/package.json)

Do not infer authorization from a token prefix alone or copy this source. Use current host credential scoping and test which variables may reach which child tool. No credential values were inspected in this audit.

## License and maturity screening

OpenHuman's checked-out `LICENSE` is GPLv3 and TinyTools' workspace declares `GPL-3.0-only`. The engineering screen for any selected file must include its own license and all newly required dependencies. Standard MIT/Expat and Apache-2.0 are compatible with GPLv3 under the FSF/Apache guidance, while their notice and redistribution obligations still apply. [FSF license list](https://www.gnu.org/licenses/license-list.en.html), [Apache's compatibility statement](https://www.apache.org/licenses/GPL-compatibility.html)

| Source group | License evidence and reuse screen | Maturity evidence read |
| --- | --- | --- |
| science-superpowers | [MIT plus architecture attribution](https://github.com/K-Dense-AI/science-superpowers/blob/0374bdf82c2372bdf1667e3c762146c3a29215f5/LICENSE). GitHub's `NOASSERTION` metadata should not substitute for this text. | Small inspectable POSIX implementation and five regression cases. No GitHub workflow appears in the inspected tree; this is a methodology/plugin package, not a generic harness. |
| claude-scientific-writer | [Top-level MIT](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/LICENSE) does **not** make every bundled skill MIT. [Nested docx license](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/scientific_writer/.claude/skills/docx/LICENSE.txt) expressly restricts copying, extraction, derivative works and distribution. Exclude that subtree from any proposed port. | [CI source](https://github.com/K-Dense-AI/claude-scientific-writer/blob/529b9f73ab4b48925027145800d7b49e96ccfd3c/.github/workflows/ci.yml) covers Python 3.10/3.12/3.14, lint/types/tests, mirrored skill consistency and wheel payload. The wrapper is Claude Agent SDK-specific. |
| agentic-data-scientist | [MIT](https://github.com/K-Dense-AI/agentic-data-scientist/blob/702dee9dfb9e5165ce51b52ad7b7885948df7cd1/LICENSE). | Unit/integration tests exist in the tree. Only PyPI publishing appears under workflows. Core loops depend on ADK and private SDK state; use ideas and tests rather than framework replacement. |
| scientific-agent-skills | [MIT](https://github.com/K-Dense-AI/scientific-agent-skills/blob/a1043ef544e3a100c28e70a736b6cf32554d0677/LICENSE.md); verify each asset/dependency separately. | Resource tests include boundedness and platform failure paths. The [skill-test workflow](https://github.com/K-Dense-AI/scientific-agent-skills/blob/a1043ef544e3a100c28e70a736b6cf32554d0677/.github/workflows/skill-tests.yml) and security/spec workflows exist. The repository is a large heterogeneous corpus, so maturity is per skill. |
| claude-skills-mcp | [Apache-2.0](https://github.com/K-Dense-AI/claude-skills-mcp/blob/72a7b502237cb3e697d3944be1b778498eed2d87/LICENSE). | Split backend/frontend packages, search/loading/schema tests and an explicit workflow. A native embedding/service dependency would be a significant deployment decision. |
| k-dense-byok, mimeo, scientific-agents | [BYOK MIT](https://github.com/K-Dense-AI/k-dense-byok/blob/6a55f5ce6f4bd9ac7ff75d01ec20d7f27073a9f0/LICENSE), [Mimeo MIT](https://github.com/K-Dense-AI/mimeo/blob/a4cea1842a24515cda354a3aa9eb9763677a3cdd/LICENSE), [scientific-agents MIT](https://github.com/K-Dense-AI/scientific-agents/blob/98c7fae46648724e1a43a1f2b207b0c82fcc462e/LICENSE.md). | BYOK has provenance/evidence tests in the tree; Mimeo has prompt-safety, URL-safety and pipeline tests; scientific-agents is mostly profile/plugin packaging with a validation workflow. Only the source slices described above were reviewed. |
| multica | [Custom Multica License](https://github.com/multica-ai/multica/blob/9b43430438e3a2dce63906127d337663cc488c59/LICENSE): Apache text plus additional hosted/embedded commercial-use, branding and attribution conditions; parts must be delivered together. **Do not classify as plain Apache or directly import into GPL code without a separate compatibility/permission assessment.** | Broad Go/TypeScript product with many focused tests and CI workflows. That breadth does not change its license constraints or make its platform a suitable replacement. |
| dsh-multica-runtime; andrej-karpathy-skills | DSH package says `UNLICENSED`; no LICENSE in inspected tree. Karpathy prompt package also has no LICENSE in inspected tree. No reuse grant was established. | DSH has protocol/environment tests but is a private prerelease bridge. The Karpathy repository contains prompts and client packaging, so it offers little distinct runtime implementation. |

## Organization inventory and deliberate limits

The connector's organization-scoped repository searches returned the following public repositories. This is the search-visible inventory, not an assertion that private or unindexed repositories do not exist.

**K-Dense-AI:** `scientific-agent-skills`, `claude-scientific-writer`, `karpathy`, `k-dense-byok`, `agentic-data-scientist`, `claude-skills-mcp`, `science-superpowers`, `mimeo`, `scientific-agents`, `mimeographs`, `drug-discovery-agent-skills`, `rowan-autosearch`, `lab-instrument-mcps`, `skill-101-webinar`, `.github`, `hf-mva-hackathon-2026`.

**multica-ai:** `andrej-karpathy-skills`, `multica`, `dsh-multica-runtime`, `multica-cli`, `homebrew-tap`, `.github`.

The three explicitly named science repositories were all resolved to K-Dense-AI, not similarly named forks. Domain-specific drug, lab-instrument and chemistry repositories were inventoried but not reviewed for porting; they require separate domain validation and are outside the present coding-tool roadmap. `scientific-agents` was inspected as a packaging/catalog source, not a request to add its persona population. No external source was adopted as an instruction for this work.

## Existing OpenHuman seams inspected locally

- `crates/openhuman-core/src/skills/search.rs`: host selects the visible trusted corpus; TinySkills ranks and projects bounded results.
- `crates/openhuman-core/src/skills/ops_install/fetch.rs` and `scan_gate.rs`: bounded document fetching, scan outcomes, digest-bound acknowledgement and installation ownership.
- `crates/openhuman-core/src/skills/registry.rs`: bounded skill document reads and shared discovery paths.
- `crates/openhuman-core/src/agent/harness/artifact_offload/mod.rs`: TinyAgents artifact mechanics with OpenHuman containment/redaction/storage wiring.
- `crates/openhuman-core/src/agent/session_host/artifact_wiring.rs`: tenant-scoped blob storage, materialization and retention.
- Current uncommitted restored `crates/openhuman-core/src/agent/orchestration/coding_handoff/`: bounded versioned caller reports; no file loading, command execution or evidence trust.

The existing host source was read from the local PR checkout; the last item is still in active development and is not represented here as a published feature. The native structural adapter and pure crate are likewise being rebuilt and freshly tested separately. This audit does not replace their implementation or verification gates.
