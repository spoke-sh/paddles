# Simplify Forensic Inspector Around Machine Narrative - SRS

## Summary

Epic: VGGIor3dC
Goal: Recast the forensic inspector as a narrative machine detail surface with an optional internals mode instead of parallel nav/list/detail panes.

## Scope

### In Scope

- [SCOPE-01] Collapse forensic selection to the shared machine-moment model.
- [SCOPE-02] Build a machine-first detail surface for selected forensic moments.
- [SCOPE-03] Move raw payloads and record metadata behind an explicit internals path.

### Out of Scope

- [SCOPE-04] Rebuilding the steering gate manifold.
- [SCOPE-05] Removing raw forensic payload access entirely.
- [SCOPE-06] Recorder/storage changes outside what the shared projection already requires.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The forensic route must adopt the same selected-turn, selected-moment, and optional internals selection model defined in voyage one. | SCOPE-01 | FR-03 | manual |
| SRS-02 | The forensic route must render a machine-first detail surface that explains the selected moment before showing raw payload internals. | SCOPE-01 | FR-03 | manual |
| SRS-03 | The route must provide an explicit internals path for raw payloads, record ids, and comparison context without keeping the old always-on nav/list/detail composition. | SCOPE-03 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The default forensic path must explain a turn with fewer concurrent controls than the current conversation/turn/record focus model. | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-02 | Forensic tests and docs must clearly distinguish the default narrative path from the explicit internals path. | SCOPE-03 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
