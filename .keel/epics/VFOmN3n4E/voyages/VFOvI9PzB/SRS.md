# Bounded Context Self-Assessment - SRS

## Summary

Epic: VFOmN3n4E
Goal: Implement bounded self-assessment for context compaction using planner evidence mechanisms, producing relevance-ranked compaction plans with artifact envelopes.

## Scope

### In Scope

- [SCOPE-01] CompactionRequest and CompactionPlan domain types
- [SCOPE-02] Bounded self-assessment: evaluate context artifacts for relevance using planner evidence mechanisms
- [SCOPE-03] Compacted summaries as artifact envelopes with locators to pre-compaction content

### Out of Scope

- [SCOPE-04] Automatic compaction triggers
- [SCOPE-05] Multi-tier compaction cascading
- [SCOPE-06] Changing existing fixed character limits

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | CompactionRequest with target_scope (Vec of artifact references), relevance_query (context summary), and budget (CompactionBudget with max_steps) | SCOPE-01 | FR-01 | test |
| SRS-02 | CompactionPlan with decisions: Vec<CompactionDecision> where each is Keep { priority }, Compact { summary }, or Discard { reason } | SCOPE-01 | FR-02 | test |
| SRS-03 | assess_context_relevance() takes CompactionRequest and returns CompactionPlan | SCOPE-02 | FR-03 | test |
| SRS-04 | Assessment respects CompactionBudget.max_steps constraint | SCOPE-02 | FR-05 | test |
| SRS-05 | Compacted summaries wrapped in ArtifactEnvelope with ContextLocator to source | SCOPE-03 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Assessment budget strictly bounded — cannot exceed allocated max_steps | SCOPE-02 | NFR-01 | test |
| SRS-NFR-02 | Compacted output is a valid input for subsequent compaction rounds | SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
