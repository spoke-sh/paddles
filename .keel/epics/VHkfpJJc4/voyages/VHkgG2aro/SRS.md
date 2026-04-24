# Implement Governed External Capability Broker - SRS

## Summary

Epic: VHkfpJJc4
Goal: Replace the noop external capability broker with local-first governed web, MCP, and connector capability execution that returns typed evidence to the recursive planner.

## Scope

### In Scope

- [SCOPE-01] Implement real external capability brokering for web, MCP, and connector-style tools through existing typed capability contracts, including catalog, governance, evidence, and projection behavior.

### Out of Scope

- [SCOPE-11] Mandatory network dependency for local-first sessions.
- [SCOPE-12] Full implementation of every connector provider in the first slice.
- [SCOPE-10] Bypassing execution governance for external tools.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Replace the noop broker with a domain-port-backed broker registry that exposes declared capability availability. | SCOPE-01 | FR-01 | test: broker catalog and unavailable-state tests |
| SRS-02 | Route external capability calls through governance before execution and return typed denial or approval evidence. | SCOPE-01 | FR-01 | test: denied, allowed, and degraded execution tests |
| SRS-03 | Append external capability results to recursive evidence and projection events without weakening local-first defaults. | SCOPE-01 | FR-01 | test: planner-loop evidence and event projection tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | External capabilities remain opt-in, declared, and unavailable by default when configuration or credentials are absent. | SCOPE-01 | NFR-01 | test: default local-first posture |
| SRS-NFR-02 | Every result includes provenance suitable for replay and trace inspection. | SCOPE-01 | NFR-04 | test: provenance serialization |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
