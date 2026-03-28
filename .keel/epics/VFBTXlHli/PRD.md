# Context Gathering Subagent Interface - Product Requirements

## Problem Statement

Paddles only has one active answer/runtime path, so retrieval-heavy requests cannot be routed to a dedicated context-gathering model such as Chroma Context-1 behind a proper subagent boundary.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Introduce a dedicated context-gathering lane for retrieval-heavy requests without replacing the default answer runtime. | Retrieval-heavy requests can be routed through a gatherer lane before final synthesis | Verified CLI and test proofs |
| GOAL-02 | Preserve the existing local-first answer and tool path for direct chat and deterministic workspace actions. | Non-retrieval turns continue to use the current local answer path with no behavior regression | Verified CLI parity |
| GOAL-03 | Make Context-1 adoption explicit, gated, and fallback-safe rather than a silent runtime swap. | Context-1 capability state is surfaced clearly and unsupported harness states fail closed | Verified config/docs and runtime proofs |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer or agent running `paddles` inside a workspace with mixed chat, coding, and retrieval-heavy prompts. | A runtime that chooses the right model lane for the request instead of forcing one model to do everything poorly. |

## Scope

### In Scope

- [SCOPE-01] Define a typed context-gathering subagent contract and evidence bundle that can sit between routing and answer synthesis.
- [SCOPE-02] Prepare runtime/model configuration for distinct gatherer and synthesizer lanes.
- [SCOPE-03] Route retrieval-heavy requests through the gatherer lane while preserving the existing default answer/tool path for other turns.
- [SCOPE-04] Add an experimental Context-1 adapter boundary, capability gate, and operator-visible fallback behavior.
- [SCOPE-05] Update planning and architecture docs to describe the lane split and specialized-model routing contract.

### Out of Scope

- [SCOPE-06] Replacing the default answer model with Context-1 or making Context-1 the primary conversation runtime.
- [SCOPE-07] Shipping Chroma's private harness behavior inside Paddles.
- [SCOPE-08] Introducing silent remote fallback for prompt execution or tool execution.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must define a typed context-gathering interface that accepts a retrieval request and returns a ranked evidence bundle plus capability metadata. | GOAL-01, GOAL-03 | must | The gatherer lane needs a stable contract before any specialized model can plug into it. |
| FR-02 | Paddles must support separate gatherer and synthesizer runtime/model lanes in configuration and service wiring. | GOAL-01, GOAL-02 | must | The architecture must stop assuming one active model path for every request. |
| FR-03 | Retrieval-heavy requests must be classified and routed through context gathering before the final answer is synthesized. | GOAL-01 | must | This is the core end-to-end behavior the epic is trying to unlock. |
| FR-04 | Direct chat, coding, and deterministic workspace-action turns must preserve the current default answer/tool path when context gathering is unnecessary or unavailable. | GOAL-02 | must | Specialized routing cannot degrade the common-case local workflow. |
| FR-05 | Context-1 support must be exposed as an explicit experimental gatherer capability with clear unsupported/harness-required states. | GOAL-03 | must | Paddles should not pretend Context-1 is drop-in compatible when its harness is specialized. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | The default answer/tool path must remain local-first and operational with no new mandatory network dependency for common prompt handling. | GOAL-02 | must | Preserves a core project invariant while the new lane is introduced. |
| NFR-02 | Routing decisions, gatherer capability state, and evidence bundle summaries must be inspectable in verbose/debug output. | GOAL-01, GOAL-03 | should | Operators need to understand why a request did or did not use context gathering. |
| NFR-03 | Unsupported specialized gatherer providers must fail closed with clear operator messaging rather than silently downgrading or swapping runtimes. | GOAL-03 | must | Prevents misleading behavior and makes harness gaps explicit. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Gatherer contract | Tests and code review of typed request/result surfaces | Story evidence for contract and integration points |
| Routing behavior | CLI/manual proofs plus targeted tests for retrieval-heavy vs direct turns | Story evidence showing lane selection and fallback behavior |
| Context-1 boundary | Config/docs review and runtime proofs for capability gating | Story evidence showing explicit unsupported/harness-required states |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Retrieval-heavy requests in Paddles can be distinguished well enough from direct answer/tool turns to justify a separate gatherer lane. | Routing may need a more explicit operator control or stronger classifier. | Validate during routing story proofs. |
| A typed evidence bundle can capture enough signal for a synthesizer model without reproducing Chroma's private harness internals. | Context-1 integration may remain documentation-only until the public harness matures. | Validate during contract and adapter-boundary stories. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Retrieval-heavy intent classification may need explicit heuristics before a small local model can cooperate reliably. | Operator | Open |
| Context-1 may remain BF16/harness-bound for near-term local setups, limiting practical adoption on constrained hardware. | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Paddles has a documented context-gathering subagent contract and evidence bundle that can be consumed by a separate answer model.
- [ ] Retrieval-heavy requests can be routed through a dedicated gatherer lane while direct chat/tool turns preserve the existing local-first path.
- [ ] Context-1 has an explicit adapter boundary and capability gate rather than being treated as a drop-in replacement for the answer runtime.
<!-- END SUCCESS_CRITERIA -->
