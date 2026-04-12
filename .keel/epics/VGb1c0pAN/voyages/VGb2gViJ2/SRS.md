# Govern Local Hands With Explicit Execution Policy - SRS

## Summary

Epic: VGb1c0pAN
Goal: Harden shell and adjacent execution hands with explicit sandbox, approval, and escalation semantics that remain visible in the recursive trace.

## Scope

### In Scope

- [SCOPE-01] Define domain/runtime contracts for sandbox posture, approval policy, permission requirements, and escalation outcomes.
- [SCOPE-02] Route shell and workspace-edit hands through a shared permission gate that fails closed when policy is insufficient.
- [SCOPE-03] Support structured deny and escalation outcomes, including bounded reuse metadata for approved exceptions.
- [SCOPE-04] Record execution-governance decisions as typed runtime and trace artifacts that UI surfaces can project.
- [SCOPE-05] Document the resulting execution-governance model, capability downgrades, and verification posture.

### Out of Scope

- [SCOPE-05] Hosted or remote sandbox infrastructure.
- [SCOPE-06] MCP, connector, or web-search tool integration beyond the governance hooks they will later reuse.
- [SCOPE-07] Final UX polish for every approval interaction across every future surface.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define explicit execution-governance contracts for sandbox mode and approval policy selection at turn start. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Shell and workspace-edit hands must declare required permissions and execute through one shared permission gate before side effects occur. | SCOPE-02 | FR-02 | manual |
| SRS-03 | When policy blocks execution, the permission gate must return a structured deny or escalation outcome instead of silently retrying with broader authority. | SCOPE-02, SCOPE-03 | FR-03 | manual |
| SRS-04 | Escalation outcomes must be able to scope additional permissions or bounded reuse metadata without permanently widening later execution. | SCOPE-03 | FR-04 | manual |
| SRS-05 | Execution-governance posture and per-action outcomes must be emitted as typed runtime and trace artifacts that operator-facing surfaces can render. | SCOPE-04 | FR-05 | manual |
| SRS-06 | Capability or profile downgrade must disable unsupported governance features honestly and document the resulting degraded posture. | SCOPE-05 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The governance model must preserve local-first execution and compose with existing harness profiles and hand contracts. | SCOPE-01, SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | Any policy-evaluation failure must fail closed with explicit diagnostics rather than widening execution authority implicitly. | SCOPE-02, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-03 | Governance posture and outcomes must remain replayable and legible across transcript, trace, and API projections. | SCOPE-03, SCOPE-04 | NFR-03 | manual |
| SRS-NFR-04 | Introducing policy metadata or escalation flows must not expand secret or credential reachability. | SCOPE-01, SCOPE-02 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
