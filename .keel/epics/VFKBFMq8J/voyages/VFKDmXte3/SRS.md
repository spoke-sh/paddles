# Trace DAG Visualization - SRS

## Summary

Epic: VFKBFMq8J
Goal: Deliver trace graph endpoint and hexagonal railroad SVG visualization

## Scope

### In Scope

- [SCOPE-01] GET /sessions/:id/trace/graph endpoint returning nodes, edges, and branches as JSON
- [SCOPE-02] Browser SVG visualization rendering the DAG as a railroad diagram
- [SCOPE-03] Hexagonal node shapes with color and icon conveying TraceRecordKind
- [SCOPE-04] Real-time updates as new TraceRecords arrive via SSE
- [SCOPE-05] Branch swimlanes showing divergence and merge points

### Out of Scope

- [SCOPE-06] Historical trace browsing across sessions
- [SCOPE-07] Interactive node editing or annotation
- [SCOPE-08] 3D or physics-based graph layouts

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Trace graph endpoint converts TraceReplay into a flat node/edge/branch JSON structure. | SCOPE-01 | FR-01 | manual |
| SRS-02 | SVG visualization renders hexagonal nodes positioned in a vertical railroad-style flow. | SCOPE-02,SCOPE-03 | FR-02 | manual |
| SRS-03 | Node color and label reflect TraceRecordKind (root, action, tool, checkpoint, merge). | SCOPE-03 | FR-03 | manual |
| SRS-04 | Branch divergence renders as parallel swimlanes splitting from the mainline. | SCOPE-05 | FR-04 | manual |
| SRS-05 | Merge records render as lanes converging back to a single line. | SCOPE-05 | FR-05 | manual |
| SRS-06 | Visualization updates in real time as new TraceRecords arrive via SSE. | SCOPE-04 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Visualization is embedded in the same HTML served by paddles with no separate build step. | SCOPE-02 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
