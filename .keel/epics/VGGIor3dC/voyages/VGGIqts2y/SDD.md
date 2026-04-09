# Simplify Forensic Inspector Around Machine Narrative - Software Design Description

> Recast the forensic inspector as a narrative machine detail surface with an optional internals mode instead of parallel nav/list/detail panes.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns the forensic inspector from a raw-record browser into a narrative explanation surface. The user should first understand the selected machine moment, its steering context, and any before/after artifact meaning. Raw payloads remain available, but only through an explicit internals path.

## Context & Boundaries

In scope is the forensic route and its supporting selectors/components. The route should reuse the shared machine-moment model and avoid preserving the older multi-pane selection model by default.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Shared machine-moment projection | Internal | Supplies selected moment semantics and raw evidence links | voyage one |
| Forensic projections | Internal | Provide payloads, steering signals, and artifact lineage for the detail surface | existing |
| Forensic route tests | Internal | Guard the new narrative path and internals path | existing |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Default forensic mode | Machine-first narrative detail | The user complaint is about incoherent narrative, not lack of raw bytes. |
| Raw payload access | Explicit internals mode | Preserves debugging depth without forcing it into the first-read path. |
| Legacy panes | Retire nav/list/detail when the machine narrative can replace them | Multiple parallel panes are a primary source of current overload. |

## Architecture

Shared machine moments + forensic payload links -> forensic route -> machine detail drawer -> optional internals panel.

## Components

- Forensic machine surface: renders selected moment context and current/baseline comparisons where relevant.
- Forensic detail drawer: explains why the selected moment mattered.
- Internals toggle/panel: reveals raw payloads, ids, and evidence anchors only on demand.

## Interfaces

- Route state: selected moment id, show internals flag.
- Detail contract: selected moment summary, steering forces, comparison context, raw payload links.

## Data Flow

1. Forensic route selects a moment from the shared machine model.
2. Detail drawer renders narrative explanation and any comparison context.
3. If the operator enables internals, raw payloads and ids appear in a bounded view.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Selected moment has no payload or comparison material | Missing linked record data | Render the narrative explanation without comparison/internals blocks | Preserve the machine story and keep raw links explicit when they exist |
| Route still depends on older selection modes to reach required detail | Failing route tests or missing operator path | Promote missing data into the shared moment contract or internals panel | Remove mode-specific dependencies rather than restoring old pane structure |
