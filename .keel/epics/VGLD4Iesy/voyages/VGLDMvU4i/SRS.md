# Add Adaptive Harness Profiles And Specialist Brains - SRS

## Summary

Epic: VGLD4Iesy
Goal: Replace stale provider-shaped heuristics with explicit harness profiles, session-queryable context, and optional specialist brains that preserve the recursive core.

## Scope

### In Scope

- [SCOPE-01] Explicit harness-profile contract for steering, compaction, and recovery policy
- [SCOPE-02] Session-queryable context slices for adaptive replay and non-destructive compaction
- [SCOPE-03] Optional specialist brains modeled as bounded session-scoped capabilities
- [SCOPE-04] Verification and docs for cross-model generalization behavior

### Out of Scope

- [SCOPE-05] Replacing the recursive planner/controller core loop
- [SCOPE-06] Hosted orchestration or non-local product surfaces
- [SCOPE-07] General plugin-marketplace work unrelated to the harness-profile model

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The runtime defines explicit harness profiles that control steering, compaction, and recovery semantics without hard-coding behavior to provider names. | SCOPE-01 | FR-05 | story:VGLDQBFqJ |
| SRS-02 | The session exposes queryable context slices for adaptive replay, rewind, and compaction-oriented retrieval outside the active prompt window. | SCOPE-02 | FR-05 | story:VGLDQBYqH |
| SRS-03 | Optional specialist brains plug into the same session and capability contracts without bypassing the recursive planner/controller architecture. | SCOPE-03 | FR-06 | story:VGLDQCIrB |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Profile and specialist-brain changes remain observable through existing trace and UI projection surfaces. | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-02 | The adaptive layer retires stale policies intentionally through explicit tests/docs rather than invisible heuristics. | SCOPE-02 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
