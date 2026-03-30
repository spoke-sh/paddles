# Step Timing Implementation - SRS

## Summary

Epic: VFNccFj7d
Goal: Deliver end-to-end step timing baselines: reservoir, persistence, classification, and colored rendering

## Scope

### In Scope

- [SCOPE-01] Per-event-type timing reservoir data structure with bounded window
- [SCOPE-02] JSON persistence to ~/.cache/paddles/ with load-at-boot and flush-after-turn
- [SCOPE-03] Pace classification (fast/normal/slow) from reservoir percentiles
- [SCOPE-04] Colored delta text rendering in transcript rows

### Out of Scope

- [SCOPE-05] Per-model or per-provider segmentation of baselines
- [SCOPE-06] User-configurable thresholds or percentile tuning
- [SCOPE-07] Aggregate analytics or timing dashboards

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Record each TurnEvent delta into a bounded per-key reservoir (VecDeque, cap 50) | SCOPE-01 | FR-01 | test |
| SRS-02 | Evict oldest entry when window is full | SCOPE-01 | FR-01 | test |
| SRS-03 | Compute nearest-rank percentile (p50, p85) from reservoir window | SCOPE-01 | FR-03 | test |
| SRS-04 | Return None for percentiles when fewer than 5 samples exist | SCOPE-01 | FR-05 | test |
| SRS-05 | Serialize reservoir to JSON and deserialize on load | SCOPE-02 | FR-02 | test |
| SRS-06 | Create cache directory on first write if missing | SCOPE-02 | FR-02 | test |
| SRS-07 | Treat missing or corrupt cache file as empty reservoir | SCOPE-02 | FR-02 | test |
| SRS-08 | Classify delta as Fast (< p50), Normal (p50–p85), or Slow (> p85) | SCOPE-03 | FR-03 | test |
| SRS-09 | Classify as Normal when insufficient history exists | SCOPE-03 | FR-05 | test |
| SRS-10 | Render delta text with pace-differentiated color | SCOPE-04 | FR-04 | manual |
| SRS-11 | Palette includes pace styles for both light and dark themes | SCOPE-04 | FR-04 | test |
| SRS-12 | Derive event type key from TurnEvent variant serde tag | SCOPE-01 | FR-01 | test |
| SRS-13 | Load reservoir at TUI startup | SCOPE-02 | FR-02 | manual |
| SRS-14 | Flush reservoir to disk after turn completion | SCOPE-02 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Reservoir window capped at 50 entries per event type | SCOPE-01 | NFR-01 | test |
| SRS-NFR-02 | No new crate dependencies | SCOPE-01 | NFR-02 | test |
| SRS-NFR-03 | File I/O does not block the UI render loop | SCOPE-02 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
