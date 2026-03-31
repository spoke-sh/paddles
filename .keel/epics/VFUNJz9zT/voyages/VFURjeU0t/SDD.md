# Evidence-Threshold Context Refinement - Software Design Description

> GOAL-01: Evidence-threshold refinement

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds adaptive interpretation refinement inside an active planner loop.
When evidence signals indicate the current interpretation context is stale, the runtime
computes whether a refinement trigger has fired, applies a policy-specific update, and
surfaces a `RefinementApplied` turn event through the existing trace stream.
Guardrails enforce cooldown and oscillation checks so context updates remain bounded.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

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

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

The voyage adds one cross-cutting refinement path to the planner loop:

1. The planner loop monitors evidence signals and evaluates `RefinementPolicy`.
2. The runtime converts policy outcomes into a refinement decision.
3. A policy gate applies cooldown and oscillation checks before state mutation.
4. Approved refinements update interpretation context and emit a structured trace event.
5. The trace stream receives the event so UI and replay tooling can observe and reason about refinements.

This keeps the existing execution boundary intact while introducing a new policy-driven
decision layer with deterministic guardrails.

## Components

### Refinement Policy

- Purpose: define when and how interpretation refinement should be considered during a turn.
- Interface: policy configuration and trigger objects consumed by the planner loop.
- Behavior: expose stable trigger types and thresholds used by the execution controller.

### Refinement Evaluator

- Purpose: evaluate active trigger/policy against current loop state and evidence pressure.
- Interface: planner loop checkpoint input -> refinement decision output.
- Behavior: return a structured decision and reason code only when policy and guard checks pass.

### Refinement Guard

- Purpose: prevent instability from repeated context churn.
- Interface: refinement decision -> cooldown and oscillation checks.
- Behavior: deny refinement when guard conditions fail and emit warning telemetry.

### Trace Event Emitter

- Purpose: record refinement outcomes.
- Interface: planner controller -> trace stream event publisher.
- Behavior: emit `RefinementApplied` records with context delta metadata.

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

```text
User Signal/Events
   │
   ▼
Planner loop checkpoint
   │
   ▼
Refinement trigger eval (RefinementPolicy)
   │
   ├─ if trigger inactive: continue normal loop
   └─ if trigger active: run oscillation/cooldown guard
         │
         ├─ blocked: emit warning telemetry
         └─ allowed: apply context refinement
               │
               └─ emit RefinementApplied turn event to trace stream
```

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
