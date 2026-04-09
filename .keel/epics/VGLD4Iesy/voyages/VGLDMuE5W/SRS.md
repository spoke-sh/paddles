# Define Durable Session And Capability Interfaces - SRS

## Summary

Epic: VGLD4Iesy
Goal: Make the session and capability surfaces the stable interfaces around the recursive harness so provider improvements do not force harness rewrites.

## Scope

### In Scope

- [SCOPE-01] Durable session semantics for wake, replay, checkpoint, and selective event-slice interrogation
- [SCOPE-02] Default recorder posture for the runtime and its migration away from optional noop recording
- [SCOPE-03] Capability negotiation for shared planning, rendering, and tool-call behavior
- [SCOPE-04] Trace and documentation updates needed to make the new interfaces inspectable

### Out of Scope

- [SCOPE-05] Execution-hand lifecycle refactors for workspace editor, terminal, or transports
- [SCOPE-06] Credential-isolation mechanics beyond the session/capability contract surface
- [SCOPE-07] Harness-profile tuning and specialist-brain orchestration

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The runtime defines a durable session contract with explicit wake, replay, checkpoint, and selective slice semantics outside the active model context window. | SCOPE-01 | FR-01 | story:VGLDQ7pnQ |
| SRS-02 | The default runtime recorder posture promotes an embedded persistent session spine while preserving bounded local-first failure behavior. | SCOPE-02 | FR-01 | story:VGLDQ8NnO |
| SRS-03 | Shared planning/rendering/tool-call behavior resolves from negotiated capabilities rather than provider-name branching wherever the behavior is conceptually the same. | SCOPE-03 | FR-02 | story:VGLDQ92nP |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Session and capability refactors remain local-first and replay-visible through the existing trace/forensic surfaces. | SCOPE-01 | NFR-01 | manual |
| SRS-NFR-02 | Operator-facing planner/synthesis behavior stays backward-compatible while the underlying session and capability contracts migrate. | SCOPE-02 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
