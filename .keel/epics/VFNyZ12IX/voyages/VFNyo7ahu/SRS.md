# Search Progress Implementation - SRS

## Summary

Epic: VFNyZ12IX
Goal: Unblock the sift search call, emit progress events, and render them in the TUI

## Scope

### In Scope

- [SCOPE-01] Move search_autonomous to a blocking thread with progress channel
- [SCOPE-02] TurnEvent for search progress with phase and elapsed time
- [SCOPE-03] TUI rendering of search progress with in-place updates
- [SCOPE-04] Upstream sift progress callback requirements document

### Out of Scope

- [SCOPE-05] Implementing the sift-side callback API
- [SCOPE-06] Search cancellation
- [SCOPE-07] Percentage-based progress bars

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Wrap search_autonomous in tokio::task::spawn_blocking with mpsc progress channel | SCOPE-01 | FR-01 | manual |
| SRS-02 | Blocking thread sends periodic elapsed-time heartbeats (~2s interval) | SCOPE-01 | FR-02 | manual |
| SRS-03 | TurnEvent::GathererSearchProgress variant with phase, elapsed_seconds, and detail fields | SCOPE-02 | FR-02 | manual |
| SRS-04 | event_type_key returns "gatherer_search_progress" and min_verbosity is 0 | SCOPE-02 | FR-02 | manual |
| SRS-05 | format_turn_event_row renders progress with phase and elapsed time | SCOPE-03 | FR-03 | manual |
| SRS-06 | Progress rows update in-place in the live tail, replaced by GathererSummary on completion | SCOPE-03 | FR-03 | manual |
| SRS-07 | Upstream sift progress callback requirements documented as bearing or ADR | SCOPE-04 | FR-05 | manual |
| SRS-08 | Document specifies callback shape, progress phases, phase data, and integration seam | SCOPE-04 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Progress events emitted at most every 2 seconds | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | No performance regression from spawn_blocking wrapper | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-03 | Graceful fallback: elapsed timer works without sift callbacks | SCOPE-01 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
