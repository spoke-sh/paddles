# Context Pressure Tracking And Events - SRS

## Summary

Epic: VFOmVwP8l
Goal: Track context truncation during assembly and emit pressure signals as turn events so the system and operators can observe context quality degradation.

## Scope

### In Scope

- [SCOPE-01] ContextPressure domain type with pressure level, truncation count, and contributing factors
- [SCOPE-02] Truncation tracking: count and categorize truncation events during context assembly
- [SCOPE-03] TurnEvent::ContextPressure for visibility in the turn event stream
- [SCOPE-04] TUI rendering of context pressure at verbose=1+

### Out of Scope

- [SCOPE-05] Automatic planner strategy adaptation based on pressure
- [SCOPE-06] Staleness detection based on temporal decay
- [SCOPE-07] Cross-turn pressure trend analysis

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | ContextPressure struct with level: PressureLevel, truncation_count: usize, factors: Vec<PressureFactor> | SCOPE-01 | FR-01 | test |
| SRS-02 | PressureLevel enum: Low, Medium, High, Critical | SCOPE-01 | FR-01 | test |
| SRS-03 | PressureFactor enum: MemoryTruncated, ArtifactTruncated, ThreadSummaryTrimmed, EvidenceBudgetExhausted | SCOPE-01 | FR-02 | test |
| SRS-04 | PressureTracker accumulates truncation events during context assembly | SCOPE-02 | FR-03 | test |
| SRS-05 | PressureLevel computed from factor count: 0=Low, 1-2=Medium, 3-5=High, 6+=Critical | SCOPE-01 | FR-04 | test |
| SRS-06 | TurnEvent::ContextPressure variant emitted after interpretation context assembly | SCOPE-03 | FR-05 | manual |
| SRS-07 | format_turn_event_row renders ContextPressure at verbose=1 as compact pressure summary | SCOPE-04 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Pressure tracking adds no measurable overhead to context assembly | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Pressure signals are informational — do not alter turn flow | SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
