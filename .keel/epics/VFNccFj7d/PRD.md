# Step Timing Reservoir - Product Requirements

## Problem Statement

Turn event deltas are displayed uniformly in the transcript with no sense of whether a step was fast or slow relative to historical norms, making it hard for users to spot bottlenecks during the wait.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Users can visually distinguish fast, normal, and slow steps in the transcript | Delta text renders in differentiated colors based on historical baselines | Correct classification after 2+ sessions |
| GOAL-02 | Baselines adapt to the user's actual environment over time | Reservoir reflects real observed durations, not hardcoded thresholds | p50/p85 computed from last 50 observations per event type |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Interactive operator | Developer using paddles interactively, waiting for turns | Know where time is being spent without reading every delta manually |

## Scope

### In Scope

- [SCOPE-01] Per-event-type timing reservoir with fixed-size window
- [SCOPE-02] Persistence to ~/.cache/paddles/step_timing.json
- [SCOPE-03] Fast/normal/slow classification using p50/p85 percentiles
- [SCOPE-04] Colored delta text in transcript rendering

### Out of Scope

- [SCOPE-05] Aggregate analytics or dashboards
- [SCOPE-06] Per-model or per-provider segmentation of baselines
- [SCOPE-07] User-configurable thresholds

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Record each TurnEvent delta into a bounded reservoir keyed by event type | GOAL-02 | must | Provides the raw data for classification |
| FR-02 | Persist reservoir to ~/.cache/paddles/step_timing.json and load at boot | GOAL-02 | must | Baselines survive across sessions |
| FR-03 | Classify each step delta as fast (<p50), normal (p50-p85), or slow (>p85) | GOAL-01 | must | Core classification logic |
| FR-04 | Render delta text with pace-appropriate color in transcript rows | GOAL-01 | must | Visual affordance for the user |
| FR-05 | Steps with fewer than 5 historical observations render as normal | GOAL-01 | should | Avoid noisy classification from insufficient data |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Reservoir window capped at 50 entries per event type | GOAL-02 | must | Bounded storage, natural adaptation |
| NFR-02 | No new crate dependencies | - | must | Keep the dependency surface minimal |
| NFR-03 | File I/O must not block the UI thread | - | should | Flush happens after turn completion, not during render |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Reservoir persistence | Unit test: write, reload, verify round-trip | Test artifact |
| Classification accuracy | Unit test: known reservoir → expected pace for given delta | Test artifact |
| Visual rendering | Manual: run 2+ interactive sessions and observe color differentiation | Session screenshot |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] step_timing.json written after first turn and survives restart
- [ ] Delta text visually differentiates fast/normal/slow after enough history
- [ ] File size stays bounded regardless of session count
<!-- END SUCCESS_CRITERIA -->
