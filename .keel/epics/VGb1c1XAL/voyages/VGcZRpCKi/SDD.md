# Establish A Typed External Capability Fabric Substrate - Software Design Description

> Establish one typed external capability fabric for web, MCP, and connector-backed actions with evidence-first normalization and governed degradation.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces a typed external capability layer around the current
recursive harness so web search, MCP-backed tools, and connector-backed app
actions can be negotiated, invoked, and observed through one runtime contract.
The slice keeps the local-first model intact: external capability use is
optional, explicitly negotiated, and always routed through existing evidence,
trace, and execution-governance channels.

The design does not add a second controller. Instead it extends the existing
planner and runtime loop with an external capability registry, invocation path,
and evidence normalizer. Surfaces consume the same external capability and
degradation vocabulary rather than inventing bespoke rendering rules.

## Context & Boundaries

In scope are:
- typed capability descriptors for web, MCP, and connector-backed actions
- recursive-loop discovery and invocation semantics for those fabrics
- evidence and provenance normalization for external results
- composition with auth, approval, and sandbox governance
- projection and documentation of active fabrics and degraded states

Out of scope are:
- a generic plugin marketplace or arbitrary extension runtime
- cloud-hosted auth backplanes that make local operation impossible
- review-mode or multi-agent orchestration beyond consuming the fabric

```
┌────────────────────────────────────────────────────────────┐
│     This Voyage: External Capability Fabric Substrate     │
│                                                            │
│ Surface Intent -> Capability Registry -> Planner / Loop    │
│                                 ↓                          │
│                  Governance Bridge + Invocation Adapter    │
│                                 ↓                          │
│               Evidence / Trace / Transcript / Projections  │
└────────────────────────────────────────────────────────────┘
        ↑                    ↑                     ↑
      Web                MCP Servers          Connectors
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing recursive planner and runtime loop | Internal runtime | Remain the single orchestration loop for local and external work | current repo |
| Execution governance substrate from mission VGb1Xq72Y | Internal runtime | Apply auth, approval, and denial semantics to external capability calls | current repo |
| Trace, transcript, and projection pipelines | Internal runtime | Surface capability metadata, evidence, and degraded states | current repo |
| Web search, MCP, and connector integrations | External capability fabrics | Provide the first concrete capability classes | current repo / connected services |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Capability abstraction | Use one typed capability descriptor and invocation envelope across web, MCP, and connector fabrics | The planner should reason about one external tool family instead of three bespoke adapters |
| Orchestration model | Extend the existing recursive loop instead of adding a second external-tools controller | Keeps runtime state and evidence flow coherent |
| Result handling | Normalize every external result into evidence, source, and runtime items | Operators need provenance and explicit degraded state |
| Governance | Route every external call through existing approval and sandbox policy surfaces | External breadth must not bypass safety posture |
| Degradation model | Represent unavailable, denied, unauthenticated, or stale capabilities explicitly | Honest degradation is required for trust and local-first operation |

## Architecture

The voyage adds a capability fabric layer around the recursive harness:

1. A capability registry advertises typed descriptors for web, MCP, and
   connector-backed actions.
2. The planner queries that registry as part of the normal recursive action
   loop and selects an external action when appropriate.
3. An invocation coordinator resolves governance, auth, and availability before
   dispatching to the owning adapter.
4. Adapter results are normalized into evidence items, source records, and
   runtime items.
5. Trace, transcript, TUI, web, and API projections consume the same external
   capability vocabulary and degraded states.

## Components

- `ExternalCapabilityDescriptor`
  Purpose: describe one external capability class.
  Interface: planner-facing metadata contract.
  Behavior: captures capability kind, availability, auth posture, side-effect
  posture, and evidence expectations.

- `ExternalCapabilityRegistry`
  Purpose: negotiate the currently available external capability set.
  Interface: runtime lookup and planner discovery surface.
  Behavior: aggregates web, MCP, and connector-backed descriptors without
  forcing planners to know adapter-specific details.

- `ExternalInvocationCoordinator`
  Purpose: execute external actions through governance-aware orchestration.
  Interface: runtime action entry point.
  Behavior: checks policy, auth, availability, and approval before dispatching
  to an adapter and producing a typed result.

- `ExternalEvidenceNormalizer`
  Purpose: convert adapter output into evidence and provenance artifacts.
  Interface: result-normalization stage between invocation and projection.
  Behavior: emits evidence items, source lineage, runtime summaries, and
  degraded-state records.

- `ExternalCapabilityProjectionAdapters`
  Purpose: render active fabrics and external results across operator surfaces.
  Interface: transcript, TUI, web, and API projection sinks.
  Behavior: keeps the same capability and degradation vocabulary visible on
  every surface.

## Interfaces

- `list_external_capabilities() -> Vec<ExternalCapabilityDescriptor>`
- `invoke_external_capability(intent) -> ExternalCapabilityResult`
- `normalize_external_result(result) -> EvidenceBundle`
- `project_external_runtime_item(item) -> ProjectionEvent`
- `describe_external_capability_state() -> CapabilityCatalogSnapshot`

## Data Flow

1. A user request or planner branch requires current or account-backed
   information.
2. The recursive planner queries the external capability registry alongside the
   rest of the runtime context.
3. The planner selects an external capability action through the same action
   loop it uses for local work.
4. The invocation coordinator validates governance, auth, availability, and
   approval posture.
5. The owning adapter performs the external call and returns a typed result.
6. The normalizer converts that result into evidence, source lineage, and
   runtime items.
7. Trace and projection layers surface the active capability and resulting
   evidence or degradation details across operator surfaces.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Capability fabric is absent or disabled | Registry returns unavailable status | Emit explicit unavailable capability state and keep local-first flow intact | Continue with local work or request a different capability |
| Auth or connector bootstrap is missing | Governance or adapter validation fails before dispatch | Return typed unauthenticated or unavailable result without pretending the call succeeded | Operator can authenticate later and retry |
| Approval or policy denies the action | Governance layer rejects the invocation | Record the denial in runtime and evidence surfaces | Request approval through the existing governance path or choose a safer action |
| Adapter returns malformed or partial output | Result normalization cannot construct a valid evidence bundle | Surface a degraded result with diagnostics instead of raw opaque output | Repair the adapter contract and rerun focused verification |
| Projection surface lacks a custom renderer for a capability item | Projection receives an unknown capability or degraded item kind | Preserve the item in trace and show a generic summary | Extend the consuming surface without changing the underlying contract |
