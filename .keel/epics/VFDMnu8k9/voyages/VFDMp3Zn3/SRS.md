# Default Gatherer, Grounded Answers, And Action Stream - SRS

## Summary

Epic: VFDMnu8k9
Goal: Make repository questions run through explicit evidence gathering by default, require cited synthesis from that evidence, and render a Codex-style action stream as the default REPL experience.

## Scope

### In Scope

- [SCOPE-01] Route repository-question turns through the explicit gatherer boundary by default when a gatherer lane is available.
- [SCOPE-02] Tighten turn classification so the controller can distinguish casual chat, deterministic actions, repository questions, and deeper decomposition/research turns.
- [SCOPE-03] Require repository-question synthesis to consume explicit evidence bundles and cite source files by default.
- [SCOPE-04] Render a default Codex-style turn event stream for classification, retrieval, planners, tools, fallbacks, and synthesis.
- [SCOPE-05] Remove or demote hidden synthesizer-private retrieval as the primary repo-question path so visible routing matches actual runtime behavior.
- [SCOPE-06] Document and prove the new evidence-first interactive behavior.

### Out of Scope

- [SCOPE-07] Replacing the default local model family or requiring a remote reasoning backend.
- [SCOPE-08] Adding a quiet flag or silent mode for the new action stream.
- [SCOPE-09] Shipping a full-screen TUI instead of the textual action stream needed for the first pass.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Repository-question turns must route through the explicit gatherer boundary by default when a gatherer lane is available, and the operator-visible stream must show that selection. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The controller must classify at least casual, deterministic action, repository question, and decomposition/research intents and surface that classification in the turn stream. | SCOPE-02 | FR-02 | manual |
| SRS-03 | Repository-question synthesis must answer from the provided evidence bundle and include source/file citations by default. | SCOPE-03 | FR-03 | manual |
| SRS-04 | If evidence is insufficient, the final answer must say so explicitly and avoid unsupported repository claims. | SCOPE-03 | FR-04 | manual |
| SRS-05 | The REPL must render a default Codex-style turn event stream covering classification, retrieval, planner/tool work, fallbacks, and final synthesis. | SCOPE-04 | FR-05 | manual |
| SRS-06 | Any remaining synthesizer-private retrieval used for repo-question handling must either emit the same operator-visible turn events or be removed from the primary repo-question path. | SCOPE-05 | FR-06 | manual |
| SRS-07 | Foundational docs and proof artifacts must document and demonstrate the evidence-first turn model, default citations, and the default action stream. | SCOPE-06 | FR-07 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Ordinary prompt handling must remain local-first with no new mandatory remote dependency. | SCOPE-01, SCOPE-03, SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | The default action stream must stay concise and scannable for interactive use while preserving the high-signal steps an operator needs to trust the turn. | SCOPE-04 | NFR-02 | manual |
| SRS-NFR-03 | Gatherer unavailability or failure must degrade safely with an explicit fallback event and no ambiguity about which path actually answered the turn. | SCOPE-01, SCOPE-04, SCOPE-05 | NFR-03 | manual |
| SRS-NFR-04 | The event and citation contract must remain reusable across static gatherers, autonomous planners, and future gatherer providers. | SCOPE-01, SCOPE-03, SCOPE-04, SCOPE-05 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
