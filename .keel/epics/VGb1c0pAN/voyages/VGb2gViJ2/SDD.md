# Govern Local Hands With Explicit Execution Policy - Software Design Description

> Harden shell and adjacent execution hands with explicit sandbox, approval, and escalation semantics that remain visible in the recursive trace.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage establishes the first concrete execution-governance slice for the
new mission. The runtime will resolve a turn-scoped execution posture, evaluate
hand requests through a shared permission gate, and project the resulting
governance decisions into trace and UI surfaces.

The design deliberately starts with shell and workspace-edit hands because they
are the most direct sources of side effects in the current harness. The same
contracts should later extend to transport and external capability fabrics
without changing the planner-facing recursive loop.

## Context & Boundaries

In scope are:
- execution-governance domain types and runtime selection
- shared permission gating for shell and workspace-edit hands
- governance outcome projection into trace and operator surfaces

Out of scope are:
- hosted sandboxes or remote policy systems
- external tool fabrics such as MCP or connectors
- final product tuning for approval UX across every surface

```text
┌────────────────────────────────────────────────────────────┐
│                 This Voyage: Execution Governance          │
│                                                            │
│  Harness Profile / Config  ->  Governance Resolver         │
│                                   ↓                        │
│                         Shared Permission Gate             │
│                           ↙              ↘                 │
│                 Shell Hand                 Workspace Hand   │
│                           ↘              ↙                 │
│                    Trace / Runtime Governance Events       │
│                                   ↓                        │
│                    TUI / Web / API Projections             │
└────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing hand adapters | Internal runtime | Provide shell and workspace-edit execution entry points to mediate | current repo |
| Harness profile selection | Internal runtime | Supplies active turn posture and downgrade semantics | current repo |
| Trace/runtime projections | Internal runtime | Surface governance events to TUI, web, and API consumers | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Governance placement | Resolve execution posture at turn start and enforce per action through a shared gate | Keeps policy structural and replayable rather than hand-local |
| First mediated hands | Start with shell and workspace-edit execution | Highest current side-effect risk with minimal scope for a first slice |
| Failure behavior | Fail closed on incomplete policy state or unsupported permissions | Safety depends on visible denial rather than implicit widening |
| Projection model | Emit typed governance events instead of relying on free-form strings | Needed for cross-surface consistency and future external fabrics |

## Architecture

The voyage introduces a small execution-governance layer between the recursive
planner/controller loop and existing side-effecting hands.

1. A turn-scoped resolver selects the active sandbox mode and approval policy.
2. Hands declare required permissions in a typed descriptor before execution.
3. A shared gate evaluates the request against the active posture and returns a
   structured outcome.
4. Allowed requests proceed to the underlying hand adapter; denied or escalated
   requests return typed results without bypassing policy.
5. Governance posture and outcomes are projected into traces and UI surfaces.

## Components

- `ExecutionGovernanceProfile`
  Purpose: represent the active sandbox posture and approval policy for a turn.
  Interface: turn-scoped resolved state consumed by side-effecting hands.
  Behavior: records the declared execution posture and any downgrade reason.

- `PermissionRequest`
  Purpose: describe the permissions a specific hand invocation requires.
  Interface: hand-to-gate request object.
  Behavior: names the hand, required permissions, and whether escalation reuse metadata applies.

- `PermissionGate`
  Purpose: evaluate permission requests against the active posture.
  Interface: pure gate returning allow, deny, or escalate outcomes.
  Behavior: fails closed on incomplete state and never silently broadens access.

- `GovernedHandAdapter`
  Purpose: wrap existing shell/workspace execution with permission checks.
  Interface: existing hand call sites, mediated through the gate.
  Behavior: forwards allowed work and returns structured blocked outcomes.

- `GovernanceProjectionEmitter`
  Purpose: convert posture selection and outcomes into runtime/trace items.
  Interface: existing projection/event pipelines.
  Behavior: keeps TUI, web, and API surfaces aligned on the same governance vocabulary.

## Interfaces

- `resolve_execution_governance(...) -> ExecutionGovernanceProfile`
- `evaluate_permission(request, profile) -> PermissionOutcome`
- `execute_with_governance(hand_request, profile) -> GovernedExecutionResult`
- `emit_governance_event(profile_or_outcome) -> RuntimeEvent/TraceRecord`

## Data Flow

1. Turn boot resolves the active execution-governance profile.
2. A shell or workspace action is prepared with a `PermissionRequest`.
3. The shared gate evaluates the request against the profile.
4. If allowed, the underlying hand runs and the result is wrapped with
   governance metadata.
5. If denied or escalated, the runtime returns a structured blocked outcome and
   emits governance events without running the side effect.
6. Projection layers render the active posture and per-action outcomes.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unknown or unsupported permission requirement | Gate cannot map request to active posture | Deny and emit explicit governance error | Extend the permission vocabulary or adjust hand descriptor |
| Missing policy state for the turn | Resolver returns incomplete governance profile | Fail closed and emit diagnostic event | Repair profile/config selection before retrying |
| Projection layer cannot render governance event | Projection test/runtime fallback detects unknown item kind | Preserve raw event in trace and surface degraded summary | Extend projection vocabulary in the owning surface |
