# Transit Artifact Capture And Inspector Projection - SRS

## Summary

Epic: VFbXKEdWb
Goal: Make transit the exact source of truth for model exchange, context lineage, and force snapshots, then project that data into a dense web forensic inspector with a secondary interactive overview.

## Scope

### In Scope

- [SCOPE-01] Capture exact assembled context, redaction-safe provider request envelopes, raw provider responses, and rendered outputs in transit
- [SCOPE-02] Capture context lineage edges and force snapshots with contribution estimates in transit
- [SCOPE-03] Expose replay and live projection APIs for web forensic inspection with provisional and final artifact states
- [SCOPE-04] Render a dense context-lineage-first 2D inspector in the web UI
- [SCOPE-05] Render a secondary interactive overview for force/topology/shadow comparison above the precise inspector
- [SCOPE-06] Stream provisional active-turn artifacts into the web inspector and reconcile them to final state

### Out of Scope

- [SCOPE-07] TUI forensic inspector parity
- [SCOPE-08] Behavior changes to planner, gatherer, synthesis, or provider selection logic
- [SCOPE-09] External telemetry services or remote collaboration
- [SCOPE-10] Ambient/decorative visualization surfaces that are not tied to inspectable transit data

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Transit records exact assembled planner/synth/context artifacts, redaction-safe provider request envelopes, raw provider responses, and normalized/rendered outputs for inspectable model exchanges. | SCOPE-01 | FR-01 | test |
| SRS-02 | Transit records context lineage edges between conversation, turn, model call, planner step, artifacts, and resulting outputs. | SCOPE-02 | FR-02 | test |
| SRS-03 | Transit records force snapshots, including contribution estimates by source for pressure, truncation/compaction, execution/edit pressure, fallback/coercion, and budget effects. | SCOPE-02 | FR-03 | test |
| SRS-04 | The web/application layer exposes replay and live update projections for forensic transit artifacts, including provisional, superseded, and final artifact states. | SCOPE-03 | FR-04 | test |
| SRS-05 | The web UI provides a context-lineage-first dense inspector that unifies navigation across conversation, turn, model call, planner loop step, trace record, and artifact sequence. | SCOPE-04 | FR-05 | manual |
| SRS-06 | The web UI lets the operator toggle between exact raw content and format-friendly rendered views for inspectable artifacts, including provider envelopes with redacted sensitive fields. | SCOPE-04 | FR-06 | manual |
| SRS-07 | The default inspector surface shows applied forces and contribution-by-source for the current lineage selection. | SCOPE-04, SCOPE-05 | FR-07 | manual |
| SRS-08 | A secondary overview above the 2D inspector visualizes topology/force state and supports shadow comparison without replacing the precise inspector. | SCOPE-05 | FR-08 | manual |
| SRS-09 | Active turns stream provisional forensic artifacts in coherent sequence and reconcile them visibly when superseded or finalized. | SCOPE-06 | FR-09 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Transit replay is sufficient to rebuild forensic inspector state after missed live updates, without depending on UI-local repair heuristics. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | test |
| SRS-NFR-02 | Browser-exposed artifacts redact auth headers and obvious secrets while otherwise preserving exact payload bodies for forensic inspection. | SCOPE-01, SCOPE-03 | NFR-02 | test |
| SRS-NFR-03 | The dense inspector remains usable for long conversations and large artifacts through local scrolling and focused panels rather than page-level churn. | SCOPE-04, SCOPE-05 | NFR-04 | manual |
| SRS-NFR-04 | The voyage introduces no mandatory hosted service and any new visualization dependency is served locally or vendored. | SCOPE-05 | NFR-05 | review |
| SRS-NFR-05 | The voyage remains web-only and does not require matching TUI inspector changes. | SCOPE-04, SCOPE-05, SCOPE-06 | NFR-06 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
