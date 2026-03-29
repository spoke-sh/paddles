# Evidence-First Interactive Turns - Product Requirements

## Problem Statement

Paddles still answers repository questions through a weak synthesizer-side context path with no default action stream, no mandatory source citations, and no clear operator visibility into retrieval, planner, tool, or synthesis steps.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Route repository questions through an explicit evidence-gathering lane before final synthesis. | Repo-question prompts use a gatherer-backed evidence bundle by default when the gatherer lane is available | Verified CLI and test proofs |
| GOAL-02 | Make final repository answers grounded and cited by default. | Repository answers include file citations and acknowledge insufficient evidence when grounding is weak | Verified runtime and test proofs |
| GOAL-03 | Make runtime work visible by default in the REPL. | The REPL renders a Codex-style action stream for classification, retrieval, tools, and synthesis without requiring `-v` | Verified CLI and snapshot-style proofs |
| GOAL-04 | Preserve lightweight chat and deterministic action handling while improving repo-question quality. | Casual and direct action turns remain available without forced retrieval overhead or hidden routing | Verified routing proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer or agent using `paddles` interactively inside a repository. | Trustworthy repository answers with visible reasoning steps and default file citations. |
| Runtime Maintainer | The engineer evolving `paddles` routing, model lanes, and gatherer interfaces. | One explicit retrieval path and an operator event model that can be extended without duplicating behavior. |

## Scope

### In Scope

- [SCOPE-01] Route repository-question turns through the explicit gatherer boundary by default when a gatherer lane is available.
- [SCOPE-02] Tighten turn classification so `paddles` can distinguish casual chat, deterministic actions, repository questions, and deeper decomposition/research turns.
- [SCOPE-03] Require synthesizer answers for repository questions to stay grounded in evidence bundles and cite source files by default.
- [SCOPE-04] Render a default Codex-style turn event stream covering classification, retrieval, planner/tool activity, and synthesis.
- [SCOPE-05] Remove or demote hidden synthesizer-private retrieval as the primary repo-question path so operator-visible routing matches runtime reality.
- [SCOPE-06] Update foundational docs and proof artifacts to demonstrate the new evidence-first interactive behavior.

### Out of Scope

- [SCOPE-07] Replacing the local synthesizer model family or making a stronger remote model mandatory.
- [SCOPE-08] Adding a quiet flag or alternate silent UI mode for the new action stream.
- [SCOPE-09] Implementing rich TUI rendering beyond the textual action stream needed for the first observable REPL experience.
- [SCOPE-10] Replacing the experimental `context-1` boundary or changing the existing autonomous-planning mission’s core adapter contract.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The controller must route repository-question turns through an explicit gatherer boundary by default when a gatherer lane is available. | GOAL-01, GOAL-04 | must | Repo answers need a visible evidence path instead of hidden synthesizer-side retrieval. |
| FR-02 | The controller must distinguish at least casual, deterministic action, repository question, and decomposition/research intents so lane selection is predictable and explainable. | GOAL-01, GOAL-03, GOAL-04 | must | Better routing is required before retrieval quality can improve consistently. |
| FR-03 | Repository-question synthesis must consume explicit evidence bundles and include source/file citations by default in the final answer. | GOAL-02 | must | Grounded answers are the core product requirement. |
| FR-04 | When evidence is insufficient or missing, the synthesizer must say so explicitly instead of generating unsupported repository explanations. | GOAL-02, GOAL-04 | must | Small local models need a strict safety rail against plausible but ungrounded prose. |
| FR-05 | The REPL must render a Codex-style action stream by default that exposes classification, retrieval, planner/tool steps, fallbacks, and synthesis events. | GOAL-03 | must | Operators need default visibility into what `paddles` is doing each turn. |
| FR-06 | Any remaining synthesizer-internal retrieval used for repo-question handling must be surfaced through the same operator event model or retired from the primary path. | GOAL-01, GOAL-03 | should | The visible runtime story must match the real execution path. |
| FR-07 | Foundational docs and proof artifacts must describe and demonstrate evidence-first turns, default citations, and the new action stream. | GOAL-01, GOAL-02, GOAL-03 | should | The behavior shift needs durable operator guidance and verifiable proof. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The default answer and action paths must remain local-first with no new mandatory remote dependency for ordinary prompt handling. | GOAL-04 | must | Preserves a core project invariant. |
| NFR-02 | The default action stream must be concise enough for interactive use while remaining detailed enough to diagnose routing and grounding decisions. | GOAL-03 | must | Operator visibility cannot make the REPL unusable. |
| NFR-03 | Gatherer failures or unavailable gatherer lanes must degrade safely with explicit operator-visible fallback events and no silent ambiguity about what path ran. | GOAL-01, GOAL-03, GOAL-04 | must | Failures need to stay understandable and controlled. |
| NFR-04 | The citation and event-stream contract must be backend-agnostic enough to work across static gatherers, autonomous planners, and future gatherer providers. | GOAL-01, GOAL-02, GOAL-03 | should | The architecture should stay extensible. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Default gatherer routing | CLI/manual proofs plus targeted routing tests | Story evidence showing repo-question turns choose the explicit gatherer path |
| Grounded synthesis | Targeted tests and CLI proofs | Story evidence showing citations and insufficient-evidence handling |
| Action stream | REPL transcript proofs and renderer-level tests | Story evidence showing default Codex-style event output |
| Runtime coherence | Code review and routing/fallback proofs | Story evidence showing hidden retrieval is removed or surfaced consistently |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A stricter evidence-plus-citation contract can materially improve answer quality even on a small local synthesizer. | We may need a stronger default model or a more extractive answer format. | Validate during grounded-synthesis story proofs. |
| The operator prefers visibility by default more than minimal terminal output. | The action stream could be perceived as noise. | Validate through transcript design and docs. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The current local synthesizer may still produce weak prose unless the grounded synthesis contract is made strongly extractive. | Runtime maintainer | Open |
| Removing hidden synthesizer-side retrieval may surface more gatherer-missing cases than expected if the default lane wiring is incomplete. | Epic owner | Open |
| The default event stream could sprawl unless truncation and grouping rules are explicit. | Runtime maintainer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Repository-question turns use an explicit gatherer-backed evidence path by default when a gatherer lane is available.
- [ ] Repository answers include default source/file citations and acknowledge insufficient evidence instead of bluffing.
- [ ] The REPL renders a default Codex-style action stream that makes routing, retrieval, tools, and synthesis visible.
- [ ] Foundational docs and proof artifacts demonstrate the new evidence-first turn behavior end-to-end.
<!-- END SUCCESS_CRITERIA -->
