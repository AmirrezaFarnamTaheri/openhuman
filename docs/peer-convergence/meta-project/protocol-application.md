# Applying the supplied Meta-Project Protocol

This investigation applies the supplied **Meta-Project Specification v1.0.0**
as a research, decomposition, design and validation method for OpenHuman. The
[supplied protocol](supplied-meta-protocol.txt) is retained with normalized line
endings and trailing whitespace. The uploaded original has SHA-256
`4d7d075b5941b3c6402ba14840fb10651aef4470b003ed37efad32f5ac89a39a`.
The [input registry](input-registry.json) preserves **217 occurrences and 198
distinct identities**, including duplicates and malformed owner placeholders.
The [raw list](requests-2026-10-18.txt) retains their original order and spelling.

## Inspiration is open-ended

The review investigates what a source can teach or inspire, not only what can
be imported from its implementation. Code, a simple mechanism, a Markdown skill
and a naive feature are examples of possible triggers. Other triggers include
product goals, interaction patterns, domain distinctions, architectural
relationships, operational practice, missing behavior, constraints, failures,
evaluation methods and cross-domain analogies. Combinations may expose value
that no single source implements or even describes.

Source maturity and conceptual value are independent. A short specification can
motivate a complete native feature family; an extensive platform can contribute
one useful invariant. A small first increment is a way to test an idea, not a
ceiling on scope. Rich features, widgets, modules and whole planes remain valid
directions when their responsibilities and interactions can be made coherent.

Every consequential recommendation must distinguish the observed trigger from
the independently derived design. A source's omission can inspire an improvement,
but the resulting capability must not be attributed to that source. An unknown
repository name can prompt a research question; it cannot supply evidence of
implementation behavior. Inspiration opportunities are non-exhaustive, even when
the input list has been fully accounted for.

## Operational interpretation

| Protocol requirement | Concrete application |
| --- | --- |
| Simplicity and explicit contracts | Compare existing-owner extensions, independent features and new modules by responsibilities, user benefit and dependency cost. A lower file count is not itself a better design. |
| Token efficiency | Preserve scope and meaning; use a declared tokenizer and workload before claiming savings. Character count, compressed bytes and model tokens are different measurements. |
| Iterative refinement | Review source evidence, owner fit, design alternatives and failure conditions separately. Record corrections and publish reviewable increments. |
| Complete artifact registry | Resolve identity, pin commit and tree, enumerate retrievable paths and object IDs, and label inspection depth. A complete tree is not a claim to have read or executed every file. |
| Hierarchical decomposition | Explore details, methods, presets, widgets, features, modules and larger compositions. Identify coherent boundaries; do not force unrelated concepts into a fixed hierarchy. |
| Merge, split and adaptation | Compare actual overlap and distinct value. Consider direct reuse, adaptation, independent native realization and synthesis separately. Record proposed supersession separately from an implemented replacement. |
| Second-order convergence | Combine first-order findings through shared identities, contracts, lifecycle and failure behavior. Explain what the combination enables beyond its inputs. |
| Independent validation | Check source/pin correspondence separately from design and owner fit. Use behavioral tests for implemented code; source review is not execution evidence. |
| Deployment | Keep proposed, implemented, tested, consumed and released states distinct. An owner library's CI result does not certify the assembled host. |

## Corrections needed for accurate execution

The protocol's `ALL_MIT`, automatic conversion and removal of attribution are
not source facts. Record the applicable repository, package and file instruments
and preserve their notices when reusing source. A missing license is not an MIT
grant; a future conversion date is not a current blanket grant. This does not
turn licensing into a blanket rejection of inspiration: an independently
specified idea is a different route from copying restricted implementation or
text. Select the realization first, then establish the requirements relevant to
that route.

The example M1/M3 dependency table is cyclic: PromptEngine depends on
ConvergenceEngine, which depends on PromptEngine. It cannot simultaneously be
the accepted acyclic architecture. Separate contracts from implementations and
map responsibilities to existing owners where they fit. A new module is a
valid alternative when it has a distinct responsibility, lifecycle and consumers;
it must not silently duplicate another owner's state or authority.

The sample `completeness_score` is undefined. A single score also hides tradeoffs
among correctness, behavior, compatibility, usability, resource cost and
maintenance. Compare candidates on explicit dimensions and retain uncertainty.
Selecting a superior base does not establish that every unique feature can be
merged without conflicting semantics or regressions.

The values `token efficiency > 0.85`, `ambiguity score < 0.15`, convergence
thresholds and seven-week schedule have no declared baseline or measurement
procedure in the supplied text. Treat them as proposed targets, not achieved
results or delivery commitments. Branch coverage and line coverage have
different denominators. Repeatedly asking a model to review its own output is
not necessarily independent validation.

The named Karpathy quotations are reproduced as supplied source text, not
independently authenticated quotations. The final statement that no code or
value is overlooked is an aspiration, not a verification receipt. Unavailable
identities, uninspected source regions, unrun tests and unresolved questions
remain visible instead of being erased by a completion label.

## Native ownership and design alternatives

The current repository's [owner guidance](../../../AGENTS.md) is the integration
baseline. TinyTools owns shared tools, structural parsing and gated file-local
operations; TinyAgents owns harness/session/execution semantics; TinyInference
owns model requests and usage; TinyJuice owns context transforms; TinyMemory
owns persistent memory/retrieval; TinySkills owns portable skill contracts;
TinyFlows owns workflows; TinyBox owns isolated process lifecycle. Browser and
computer effects belong to TinyComputer, while protocol and service adapters
retain their corresponding TinyMCP/TinyConnectors boundaries. OpenHuman supplies
product configuration, account/workspace policy, approvals and presentation.

These mappings guide design rather than prohibiting new responsibilities.
For example, an evidence workbench may initially compose execution receipts,
source revisions and artifact links from current owners. A later evidence
module could be justified by shared schema evolution and independent consumers.
It must remain a consumer of actual outcomes rather than inventing execution
authority from retrieved text. A research campaign module could coordinate
preregistration, trials, review and publication while reusing existing execution
and storage contracts.

## Required content of a useful dossier

| Record | Questions it must answer |
| --- | --- |
| Source evidence | Which repository identity, commit, tree and exact files were inspected? Was a claim observed in implementation, stated in a README, or only inferred? |
| User problem | Who benefits, during which concrete task, and what current limitation is addressed? |
| Decomposition | What useful details, behaviors, representations, widgets, features or larger responsibilities can be distinguished? |
| Inspiration | What new native direction follows from the observation, failure, goal or analogy? Which assumptions are ours? What other combinations remain possible? |
| Owner fit | What already exists? Which contract or behavior is missing? What remains in the host and what belongs in a shared component? |
| Alternatives | Would reuse, adaptation, an independent implementation, a conformance test, a recipe or a distinct module best serve the goal? |
| Acceptance | Which observable cases distinguish success from superficial similarity? Include stale data, malformed inputs, cancellation, partial failure and misleading evidence where applicable. |
| Evidence limits | Which files, dependencies, assets, platforms and behavior were not inspected or executed? What would resolve an identity or source gap? |
| Implementation state | Is this a proposed design, an implemented owner change, a tested behavior, consumed host integration or released artifact? |

The requested sidequest is an investigation. Its proposals do not silently
become claims that hundreds of peer features have been implemented. The active
PR's onboarding, structural tools, reusable chunks, handoffs, fork composition
and CI repairs have their own
[implementation and roadmap evidence](../../features/pr14-maturation-review.md).
Research completion and software completion are different observations, and
neither substitutes for the other.
