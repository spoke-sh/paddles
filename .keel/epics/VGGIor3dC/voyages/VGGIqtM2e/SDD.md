# Build Turn Machine Stage For Transit - Software Design Description

> Replace the current transit board and observatory with a simpler machine-stage that shows how a turn moves through steps, diversions, jams, and outputs.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage reinterprets the transit surface as a single moving machine. The stage is the primary surface. The scrubber selects time. The selected moment exposes a concise causal explanation. The older split between node board and observatory becomes one interaction model.

## Context & Boundaries

In scope is the transit route only. It consumes the shared machine-moment projection defined in voyage one. It should not reinvent trace semantics or expose old transit chrome by default.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Shared machine-moment projection | Internal | Supplies machine-stage geometry and labels | voyage one |
| Transit route tests | Internal | Guards the simplified machine-stage contract | existing |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Primary surface | Use one machine stage plus bottom scrubber | This matches the manifold’s strongest interaction pattern and reduces operator branching. |
| Detail surface | Selected-moment popup/drawer explains the machine part | The user should understand cause before raw node metadata. |
| Old transit chrome | Remove or demote it when the stage already expresses the same information | Redundant controls are a major source of current overload. |

## Architecture

Shared machine moments -> transit route selectors -> transit machine stage -> selected-moment detail -> optional internals metadata.

## Components

- Transit machine stage: renders ordered machine moments and their geometry.
- Transit scrubber: selects temporal position/moment.
- Transit detail surface: explains selected moment, including diverter/jam/output semantics.

## Interfaces

- Route input: machine moments for the current turn.
- Route state: selected moment id, zoom/pan only if still needed after simplification.

## Data Flow

1. Transit route receives machine moments.
2. Moments render into the stage and scrubber.
3. User selects a moment from stage or scrubber.
4. Detail surface explains that moment in narrative terms.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| No moments are available for the current turn | Empty selector result | Render a simple empty machine state | Wait for a trace-producing turn |
| Too many fine-grained moments make the stage noisy | Visual review or route tests show overload | Group them more coarsely at the selector layer | Refine shared moment projection without exposing raw nodes directly |
