# Trace Contract And Recorder Port - SRS

## Summary

Epic: VFH4BXH4F
Goal: Define a paddles-owned trace contract and recorder boundary that can capture recursive planner turns, branches, tool activity, and artifacts in a form that later maps cleanly onto embedded transit lineage semantics.

## Scope

### In Scope

- [SCOPE-01] Define a `paddles` trace contract for roots, branches, tool activity, selection artifacts, and checkpoints.
- [SCOPE-02] Refactor planner state and turn execution data into recorder-ready structured traces.
- [SCOPE-03] Add a `TraceRecorder` port with non-storage-specific adapters.
- [SCOPE-04] Add artifact-envelope support for large turn payloads.
- [SCOPE-05] Add an embedded `transit-core` recorder adapter plus replay proof.
- [SCOPE-06] Update foundational docs to explain the recorder boundary and transit alignment.

### Out of Scope

- [SCOPE-07] Requiring a networked `transit` server for normal `paddles` runtime use.
- [SCOPE-08] Full shared multi-user trace storage, analytics, or merge-visualization UX.
- [SCOPE-09] Replacing the existing gatherer boundary or graph-mode mission with recorder work in this slice.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `paddles` must define a typed trace contract that can represent task roots, planner branches, tool request/result pairs, selection artifacts, and completion checkpoints with stable identifiers and lineage references. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Planner loop state and turn execution data must preserve structured branch and lineage information instead of string-only pending branches or renderer-only summaries. | SCOPE-02 | FR-02 | manual |
| SRS-03 | The runtime must expose a `TraceRecorder` port distinct from `TurnEventSink`, with `noop` and in-memory adapters that accept the typed trace contract. | SCOPE-03 | FR-03 | manual |
| SRS-04 | The trace contract must support artifact envelopes for large prompts, model outputs, tool outputs, and rich graph traces through logical references plus inline metadata. | SCOPE-04 | FR-04 | manual |
| SRS-05 | `paddles` must provide an embedded `transit-core` recorder adapter and replay proof that preserve lineage semantics without leaking raw `transit` types across the domain boundary. | SCOPE-05 | FR-05 | manual |
| SRS-06 | Foundational docs and proof artifacts must explain the recorder boundary, transit alignment, embedded/server distinction, and current limitations honestly. | SCOPE-06 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Recording must remain local-first, bounded, and fail closed when a recorder adapter is unavailable or storage integration fails. | SCOPE-03, SCOPE-05 | NFR-01 | manual |
| SRS-NFR-02 | The trace contract must remain generic across repositories and evidence domains rather than Keel-specific. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-03 | Operator-facing transcripts and event streams must remain concise even when durable trace data is richer than the rendered UI. | SCOPE-02, SCOPE-03, SCOPE-06 | NFR-03 | manual |
| SRS-NFR-04 | The trace and artifact contracts must stay storage-model-neutral inside `paddles` even when the first recorder adapter targets embedded `transit-core`. | SCOPE-01, SCOPE-03, SCOPE-04, SCOPE-05 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
