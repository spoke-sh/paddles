# Collapse Runtime Lane Terminology - Software Design Description

> Collapse planner, synthesizer, and gatherer lane terminology across user-facing surfaces and internal Rust code so the codebase centers on turn runtime phases and model clients.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage performs the deep terminology cleanup. It is not limited to public
copy: internal Rust code should no longer organize the runtime around planner,
synthesizer, and gatherer lanes. The target codebase names turn runtime phases
and components directly: action selection, retrieval, execution, evidence,
reflection/refinement, model clients, and final rendering.

## Context & Boundaries

The turn loop remains the center of orchestration. This voyage changes names,
module boundaries, ports, tests, prompts, CLI/config surfaces, and documentation
to match that architecture. Compatibility aliases may remain only when they are
explicit migration shims. This voyage does not remove Sift retrieval/indexing.

```
┌─────────────────────────────────────────┐
│              Turn Runtime               │
│                                         │
│  action selection -> execution          │
│  retrieval -> evidence -> refinement    │
│  final rendering -> completion          │
└─────────────────────────────────────────┘
        ↑               ↑
 model clients     tool/capability surface
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Turn loop modules | Rust application | Preserve orchestration behavior | repo-local |
| Prompt/execution contract tests | Rust tests | Preserve model-owned reasoning contract | repo-local |
| CLI/TUI/web route tests | Rust/frontend tests | Prove public terminology changed intentionally | repo-local |
| Docs | project docs | Own architecture and configuration vocabulary | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Internal cleanup scope | Include Rust code | A logical codebase requires internals to match the public architecture. |
| Runtime center | Turn loop | The loop already owns orchestration and should be the conceptual anchor. |
| Phase boundaries | Preserve behavior under clearer names | Over-flattening into one object would make tests and reasoning worse. |
| Compatibility aliases | Explicit shims only | Hidden lane concepts would keep the old architecture alive. |

## Architecture

The refactor should move from lane-shaped abstractions toward explicit turn
runtime components. The exact module names should follow local code shape, but
the target vocabulary is:

- action selection
- model client
- retrieval or evidence provider
- execution/tool governance
- evidence accumulation
- reflection/refinement
- final rendering
- turn runtime configuration/preparation

## Components

| Component | Purpose | Behavior |
|-----------|---------|----------|
| Turn runtime config/preparation | Build runtime components | Replaces lane-shaped config/preparation names |
| Action-selection phase | Choose model-directed actions | Replaces planner-lane framing |
| Final-rendering phase | Produce terminal answer | Replaces synthesizer-lane framing |
| Retrieval/evidence provider | Gather context | Replaces gatherer-lane framing where applicable |
| Compatibility shim | Support old names temporarily | Emits explicit migration warnings/errors |

## Interfaces

Public CLI/config/docs should speak in turn-runtime phase names. Internal Rust
APIs should avoid `lane` terminology for active runtime architecture. Historical
research artifacts and compatibility shims may retain old words when clearly
scoped.

## Data Flow

1. Turn runtime preferences are normalized.
2. Model clients and retrieval providers are prepared.
3. The turn loop performs action selection, execution, evidence gathering,
   refinement, and final rendering.
4. Projections, logs, docs, and UI copy present the same turn runtime vocabulary.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Rename changes behavior | Tests fail | Revert the behavior change inside the story and keep only the refactor | Add focused regression test |
| Old lane term remains in active public surface | String scans or UI tests | Update surface or mark compatibility shim explicitly | Replace with turn-runtime term |
| Compatibility shim becomes the canonical path | Config/doc review | Move shim behind migration-only parser | Persist only new names |
