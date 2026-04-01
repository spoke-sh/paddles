# Shared Conversation Transcript Plane - SRS

## Summary

Epic: VFYbtfpVG
Goal: Replace interface-local transcript assembly with a shared conversation-scoped transcript projection and use it as the transcript source for TUI and web surfaces.

## Scope

### In Scope

- [SCOPE-01] Add conversation-scoped transcript replay in the application layer over durable prompt/completion records
- [SCOPE-02] Introduce shared conversation attachment semantics for cross-surface prompt entry
- [SCOPE-03] Emit transcript update signals independently of `TurnEvent` progress telemetry
- [SCOPE-04] Migrate the web UI transcript bootstrap and live updates onto the canonical conversation plane
- [SCOPE-05] Migrate the TUI transcript bootstrap and live updates onto the canonical conversation plane

### Out of Scope

- [SCOPE-06] Trace DAG redesign or changes to step visualization styling
- [SCOPE-07] Multi-machine collaboration or remote synchronization
- [SCOPE-08] Planner/gatherer behavior changes unrelated to transcript visibility

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The application service exposes replay for a single conversation identity that returns transcript rows derived from durable prompt and completion records. | SCOPE-01 | FR-01 | test |
| SRS-02 | A prompt submitted from TUI, web, or CLI can target the same conversation identity so every attached surface observes one shared transcript. | SCOPE-02 | FR-02 | test |
| SRS-03 | Transcript update delivery is emitted through a dedicated transcript-plane signal rather than inferred from `TurnEvent` progress completion. | SCOPE-03 | FR-03 | test |
| SRS-04 | The web UI bootstraps and updates transcript rows from the canonical conversation plane and reflects turns entered from other surfaces without reload. | SCOPE-04 | FR-04 | manual |
| SRS-05 | The TUI bootstraps and updates transcript rows from the canonical conversation plane and reflects turns entered from other surfaces without restart. | SCOPE-05 | FR-05 | manual |
| SRS-06 | Progress events remain available for trace/activity display, but transcript hydration and reconciliation no longer depend on `synthesis_ready` or similar progress events. | SCOPE-03, SCOPE-04, SCOPE-05 | FR-06 | review |
| SRS-07 | Surface-specific transcript repair paths are removed or retired once the canonical conversation plane is authoritative. | SCOPE-04, SCOPE-05 | FR-07 | review |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Cross-surface transcript updates appear without manual page reload, TUI restart, or operator-triggered replay commands. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-01 | manual |
| SRS-NFR-02 | The implementation keeps `process_prompt_in_session_with_sink(...)` as the canonical turn execution path while transcript projection is unified. | SCOPE-01, SCOPE-02 | NFR-02 | review |
| SRS-NFR-03 | Conversation-scoped replay is sufficient to recover from missed update delivery without global trace scraping. | SCOPE-01, SCOPE-03 | NFR-04 | test |
| SRS-NFR-04 | The voyage introduces no new external network service or browser build dependency. | SCOPE-01, SCOPE-04, SCOPE-05 | NFR-03 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
