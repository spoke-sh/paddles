# Establish A Replayable Multi-Agent Delegation Substrate - SRS

## Summary

Epic: VGb1c2DBj
Goal: Establish one replayable multi-agent delegation substrate with explicit lifecycle operations, ownership boundaries, and parent-visible worker artifacts across shared surfaces.

## Scope

### In Scope

- [SCOPE-01] Define typed delegation lifecycle operations for spawning, follow-up input, waiting on, resuming, and closing worker agents.
- [SCOPE-02] Add explicit role metadata and ownership guidance to delegated work so parent and worker responsibilities stay visible.
- [SCOPE-03] Capture worker outputs, tool calls, and completion summaries as parent-inspectable runtime artifacts.
- [SCOPE-04] Project delegated worker state and integration status through transcript, TUI, web, and API surfaces.
- [SCOPE-05] Update docs and tests around delegation semantics, degradation rules, and verification posture.

This voyage also constrains implementation to the existing thread-lineage and
recursive runtime substrate rather than ad-hoc branch spawning.

### Out of Scope

- [SCOPE-06] Unbounded autonomous swarms, self-replicating delegation, or cloud cluster orchestration.
- [SCOPE-07] Parallel write access without explicit ownership, merge responsibility, or parent integration authority.
- [SCOPE-08] Replacing the parent recursive loop with a separate orchestration product or hidden worker state store.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define typed worker lifecycle operations for spawn, follow-up input, wait, resume, and close instead of relying on prompt-only delegation conventions. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Delegated work must carry explicit role metadata, ownership guidance, and parent integration responsibility as first-class runtime contracts. | SCOPE-02 | FR-02 | manual |
| SRS-03 | Parent and worker coordination must preserve durable thread lineage so the parent can continue non-overlapping work, then wait for or integrate worker results without losing replayability. | SCOPE-01, SCOPE-03 | FR-04 | manual |
| SRS-04 | Worker outputs, tool calls, and final summaries must be recorded as traceable artifacts that the parent can inspect and integrate. | SCOPE-03 | FR-03 | manual |
| SRS-05 | Transcript, TUI, web, and API surfaces must render one shared delegation vocabulary for active workers, roles, ownership, progress, and completion or integration state. | SCOPE-04 | FR-06 | manual |
| SRS-06 | Invalid lifecycle requests or ownership conflicts must degrade honestly with explicit status instead of silently mutating shared state or merging unsafe work. | SCOPE-02, SCOPE-04 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Delegated workers must remain bounded by the same execution-governance and evidence policies as the parent recursive harness. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | manual |
| SRS-NFR-02 | Multi-agent coordination must stay replayable and comprehensible across transcript and projection surfaces. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-02 | manual |
| SRS-NFR-03 | Ownership semantics must minimize merge conflicts and hidden shared-state mutation during delegated work. | SCOPE-02, SCOPE-03 | NFR-03 | manual |
| SRS-NFR-04 | The delegation substrate must deepen the existing recursive harness and thread-lineage model rather than spawning an unrelated orchestration subsystem. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
