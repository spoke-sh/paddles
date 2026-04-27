# Extract Planner Executor Boundaries - Software Design Description

> Move the recursive planner executor loop, planner action execution helpers, and external capability execution helpers out of the planner orchestration module while preserving behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extracts executor-side behavior from `src/application/mod.rs` into explicit application modules. Turn orchestration will call the recursive control chamber for planner execution, while helper modules own terminal/workspace execution metadata and external-capability adapter execution behavior.

## Context & Boundaries

The boundary is internal to the application layer. The refactor must not change planner decisions, execution-governance outcomes, external-capability availability, evidence ranking, or public service APIs.

```
Turn orchestration
  -> recursive_control chamber
      -> planner_action_execution module
      -> external_capability_execution module
      -> existing domain ports and infrastructure adapters
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Domain planner/execution ports | internal | Preserve existing planner and executor contracts | current crate |
| Execution governance gate | internal | Keep policy checks centralized and unchanged | current crate |
| ExternalCapabilityBroker | internal port | Invoke configured capability adapters through existing boundary | current crate |
| Keel board engine | repo tooling | Record lifecycle and verification evidence | `keel doctor` |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Module split | `recursive_control`, `planner_action_execution`, and `external_capability_execution` | Names the planner, executor, and external capability boundaries called out by the user without changing behavior. |
| API visibility | `pub(super)` helpers only | Keeps the boundary reusable inside application orchestration without widening public API. |
| Behavior changes | Forbidden | This is a modularity refactor, not a planner or executor semantic change. |

## Architecture

`src/application/mod.rs` remains the service wiring owner. `recursive_control.rs` owns the recursive planner executor loop and dispatches planner actions. `planner_action_execution.rs` owns planner action query/evidence-source mapping and governed terminal helper execution. `external_capability_execution.rs` owns capability invocation rendering, governance-gated broker invocation, outcome formatting, and evidence projection.

## Components

| Component | Purpose | Interface |
|-----------|---------|-----------|
| `recursive_control` | Recursive planner executor loop and action dispatch boundary. | `RecursiveControlChamber::execute_recursive_planner_loop`. |
| `planner_action_execution` | Executor-side helpers used by planner action dispatch. | `pub(super)` functions and result structs. |
| `external_capability_execution` | Governance-gated external capability execution adapter path. | `pub(super)` execution function, frame, and formatting helpers. |
| `MechSuitService` | Service wiring and shared runtime dependencies. | Existing public service methods unchanged. |

## Interfaces

No external API or protocol changes are planned. Internal helper paths change only inside the application module.

## Data Flow

Planner decisions still flow through turn orchestration into recursive control. When a terminal/workspace/external-capability action is selected, recursive control delegates helper work to the extracted modules, receives the same summaries/evidence, and records the same turn events.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Governance event emission changes | Existing runtime tests or focused external-capability tests fail | Rework extraction until events match | Keep test coverage and adjust module call sites |
| Evidence source or summary changes | Focused helper tests fail | Preserve old formatting | Re-run `cargo test` |
| Board lifecycle drift | `keel doctor` fails | Fix board artifacts or lifecycle state | Re-run `keel doctor` |
