# Model-Judged Interpretation And Retrieval - SRS

## Summary

Epic: VFJ5rdPZP
Goal: Replace residual reasoning heuristics with model-judged interpretation, fallback, and retrieval selection while preserving controller safety constraints.

## Scope

### In Scope

- [SCOPE-01] Remove residual legacy routing/tool-inference heuristics from remaining direct paths.
- [SCOPE-02] Replace lexical interpretation relevance scoring and hint/procedure ranking with model-judged selection.
- [SCOPE-03] Replace heuristic initial/planner fallback choice with constrained model re-decision where reasoning is required.
- [SCOPE-04] Replace retrieval/evidence heuristics that encode reasoning rather than safety.
- [SCOPE-05] Update docs and proof artifacts so the controller-versus-model boundary is explicit.

### Out of Scope

- [SCOPE-06] Removing controller budgets, safe-command allowlists, path validation, or fail-closed behavior.
- [SCOPE-07] Requiring remote models or changing the local-first runtime posture.
- [SCOPE-08] Making `AGENTS.md` root discovery configurable in this slice.
- [SCOPE-09] Rebuilding every tool surface into a fully unified resource graph.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Remaining legacy direct-path routing/tool-inference heuristics must be removed or demoted so the primary path uses constrained model-selected actions instead of string classification. | SCOPE-01 | FR-01 | automated |
| SRS-02 | Interpretation-time guidance relevance, tool hinting, and procedure selection must come from model judgement rooted at `AGENTS.md` and its referenced guidance graph rather than lexical scoring. | SCOPE-02 | FR-02 | automated |
| SRS-03 | Invalid initial-action and planner-action replies must trigger constrained model re-decision before any controller fallback that substitutes reasoning. | SCOPE-03 | FR-03 | automated |
| SRS-04 | Retrieval query/mode selection and evidence prioritization on recursive turns must stop depending on hardcoded reasoning heuristics where model judgement can decide the better path. | SCOPE-04 | FR-04 | automated/manual |
| SRS-05 | Foundational docs and proof artifacts must describe the resulting controller-versus-model boundary and the remaining controller-owned constraints. | SCOPE-05 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | All heuristic-removal changes must remain local-first, bounded, observable, and fail-closed. | SCOPE-01,SCOPE-03,SCOPE-04 | NFR-01 | automated/manual |
| SRS-NFR-02 | The replacement logic must remain generic across repositories and must not smuggle project-specific intents back into the controller. | SCOPE-01,SCOPE-02,SCOPE-04 | NFR-02 | manual |
| SRS-NFR-03 | Operator-visible traces must still distinguish model judgement from controller validation/execution after the heuristic-removal work lands. | SCOPE-05 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
