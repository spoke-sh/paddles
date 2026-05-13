# Shared Planner Schema Contract - Software Design Description

> Create the shared authored planner action schema renderer and tests proving it matches the Rust action surface.

**SRS:** [SRS.md](SRS.md)

## Overview

Introduce a shared planner action schema contract near the application/domain
boundary. The contract is authored data, not provider prompt prose. It describes
each supported action, required fields, JSON example, mutability posture, and
which planner prompt variants may render it. A renderer turns that contract into
canonical prompt blocks for initial action, recursive next action, retry, and
redecision prompts.

## Context & Boundaries

This voyage creates the contract and proves parity with Rust action enums. It
does not yet migrate Sift or HTTP prompts; that happens in the adoption voyage.

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `PlannerAction` / `InitialAction` / `WorkspaceAction` | domain enum | Source action surface to match | current crate |
| `PlannerExecutionContract` | application contract | Turn-specific availability and completion constraints | current crate |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Schema ownership | Authored shared contract plus parity tests | Keeps prompt/API surface reviewable while preventing enum drift |
| Renderer boundary | Application/domain-adjacent module, provider adapters consume rendered text | Avoids infrastructure adapter ownership of shared action vocabulary |
| Capability availability | Continue through `PlannerExecutionContract` | Availability is turn-specific; schema vocabulary is stable |

## Architecture

The shared contract exposes typed entries such as action name, required fields,
example JSON, and prompt-variant eligibility. The renderer produces named schema
blocks so tests and adapters can compare exact canonical output.

## Components

| Component | Purpose |
|-----------|---------|
| Planner action schema contract | Authored list of supported action shapes |
| Planner action schema renderer | Converts contract entries into prompt text |
| Schema parity tests | Compare contract coverage to Rust enum labels |

## Interfaces

The renderer should expose stable methods or options for initial-action and
recursive-action schema blocks. Adapter-specific transport instructions remain
outside this interface.

## Data Flow

Rust action enums define supported runtime variants. The authored schema mirrors
those variants. Tests compare both sets. Prompt builders consume rendered schema
blocks in later voyages.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Rust enum adds a variant missing from schema | parity test failure | fail CI | add schema entry or explain exclusion |
| Schema includes an unsupported action | parity test failure | fail CI | remove entry or implement runtime action |
| Renderer omits a required field | renderer unit test failure | fail CI | update contract or renderer |
