# Persist Replayable Sessions And Context - SRS

## Summary

Epic: VHkfpJJc4
Goal: Introduce durable local-first thread, rollout, snapshot, compaction, and rollback state aligned with trace projections and recursive evidence.

## Scope

### In Scope

- [SCOPE-06] Introduce durable session, rollout, snapshot, replay, compaction, and rollback foundations aligned with existing trace projections.

### Out of Scope

- [SCOPE-11] Cloud synchronization.
- [SCOPE-11] Hosted multi-tenant storage.
- [SCOPE-12] Replacing trace projections in this voyage.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Persist turn, planner, evidence, and governance records in a local-first session store. | SCOPE-06 | FR-06 | test: session persistence tests |
| SRS-02 | Record snapshots and rollback anchors for workspace-affecting actions. | SCOPE-06 | FR-06 | test: snapshot and rollback fixtures |
| SRS-03 | Support replay, fork, and compaction metadata for recursive context reconstruction. | SCOPE-06 | FR-06 | test: replay and compaction tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Storage remains local by default and portable across project workspaces. | SCOPE-06 | NFR-01 | test: default storage path tests |
| SRS-NFR-02 | Stored state is versioned for future migrations. | SCOPE-06 | NFR-04 | test: schema/version tests |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
