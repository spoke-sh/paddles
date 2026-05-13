# Planner Schema Documentation - Software Design Description

> Update foundational docs so the turn loop and planner action contract describe one shared schema renderer plus turn-specific capability manifests.

**SRS:** [SRS.md](SRS.md)

## Overview

Update foundational documentation after the implementation lands so operator
docs match runtime reality: one shared authored planner action schema renderer,
provider-specific transport wrappers, and turn-specific capability manifests.

## Context & Boundaries

This voyage is documentation-only and follows the implementation changes. It
does not alter runtime behavior.

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
| Shared schema implementation | code behavior | Source of documented truth | prior voyages |
| README / POLICY / ARCHITECTURE | foundational docs | Owning operator and architecture contracts | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Docs timing | Update docs with implementation slice | Prevents docs from describing aspirational behavior |
| Ownership | README for overview, POLICY for invariants, ARCHITECTURE for boundary | Keeps each contract in one owning document |

## Architecture

README explains the planner action surface and ReAct relation. POLICY states the
invariant forbidding adapter-local schema drift. ARCHITECTURE maps the shared
renderer into the planner/provider prompt pipeline.

## Components

| Component | Purpose |
|-----------|---------|
| README update | Operator-facing overview |
| POLICY update | Binding invariant |
| ARCHITECTURE update | Implementation map |

## Interfaces

No runtime interface changes in this voyage.

## Data Flow

Documentation follows implementation evidence from tests and code diffs.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Docs imply schema is remote-only | review failure | revise docs | point to shared renderer |
| Docs duplicate schema details incorrectly | review/test failure | revise docs | reference canonical contract instead |
