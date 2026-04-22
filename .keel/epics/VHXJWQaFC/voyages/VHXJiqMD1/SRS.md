# Normalized Deliberation Signals And Rationale Compilation - SRS

## Summary

Epic: VHXJWQaFC
Goal: Normalize provider-native reasoning into harness-safe deliberation signals and use those signals to improve refine, branch, and stop decisions while keeping paddles rationale explicit and auditable.

## Scope

### In Scope

- [SCOPE-01] A provider-agnostic `DeliberationSignals` contract for harness-safe reasoning inputs
- [SCOPE-02] Signal extractors that translate provider-native reasoning artifacts into normalized signals or explicit `none`/`unknown`
- [SCOPE-03] Recursive harness changes using normalized signals to improve branch, refine, retry, and stop decisions
- [SCOPE-04] `rationale` compilation rules and operator surfaces that show evidence and signal summaries rather than raw provider reasoning
- [SCOPE-05] Decision-path tests covering both native-continuation and no-op providers

### Out of Scope

- [SCOPE-06] Replacing paddles `rationale` with provider-native reasoning text
- [SCOPE-07] Default transcript/stream display of raw provider-native reasoning
- [SCOPE-08] Unrelated planner prompt tuning or UI redesign

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The application layer must define a provider-agnostic `DeliberationSignals` contract that can carry continuation, uncertainty, evidence-gap, branching, risk, and stop-related hints without referencing provider payload shapes. | SCOPE-01 | FR-05 | test |
| SRS-02 | Providers may contribute zero or more normalized deliberation signals, and absence of signals must be represented explicitly and safely. | SCOPE-02 | FR-05 | test |
| SRS-03 | The recursive harness must be able to use normalized deliberation signals to improve branch, refine, retry, and stop decisions. | SCOPE-03 | FR-06 | test |
| SRS-04 | Final planner or synthesis decisions must persist a concise paddles `rationale` derived from chosen action, evidence, and normalized signals rather than raw provider reasoning content. | SCOPE-04 | FR-07 | review |
| SRS-05 | Transcript, manifold, and forensic/operator surfaces must show rationale and signal boundaries without exposing raw provider-native reasoning by default. | SCOPE-04 | FR-07 | review |
| SRS-06 | The repository must include decision-path tests covering at least one native-continuation provider and one explicit no-op provider. | SCOPE-05 | FR-06 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Normalized deliberation signals must remain stable and provider-agnostic enough that harness logic can consume them without matching on provider names. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-05 | review |
| SRS-NFR-02 | Signal and rationale surfaces must remain bounded and auditable; they must not devolve into proxy storage for raw provider-native reasoning. | SCOPE-04 | NFR-02 | review |
| SRS-NFR-03 | Decision-path tests must remain deterministic enough to catch regressions in signal handling and rationale separation. | SCOPE-05 | NFR-04 | test |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
