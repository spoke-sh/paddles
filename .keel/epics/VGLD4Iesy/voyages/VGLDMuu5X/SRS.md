# Decouple Brain From Hands In The Local Runtime - SRS

## Summary

Epic: VGLD4Iesy
Goal: Decouple the recursive brain from local execution hands so workspace tools, transports, and future runtimes can fail, recover, and swap independently.

## Scope

### In Scope

- [SCOPE-01] Shared execution-hand lifecycle and diagnostics vocabulary
- [SCOPE-02] Workspace editor and terminal runner migration onto that hand contract
- [SCOPE-03] Credential mediation for transport- and tool-facing local execution paths
- [SCOPE-04] Trace and diagnostic visibility for hand state and recovery events

### Out of Scope

- [SCOPE-05] Durable session and capability-negotiation contracts already defined in voyage one
- [SCOPE-06] Harness-profile tuning and specialist-brain orchestration
- [SCOPE-07] Hosted or IDE-fed execution environments

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The runtime defines a shared execution-hand contract covering lifecycle, provisioning, execution, recovery, and diagnostics across local action surfaces. | SCOPE-01 | FR-03 | story:VGLDQ9Zoe |
| SRS-02 | Workspace editing and terminal execution run through the shared hand contract without breaking existing local-first edit semantics. | SCOPE-02 | FR-03 | story:VGLDQAMpx |
| SRS-03 | Credentials and privileged transport/tool state are mediated so generated code and shell execution do not receive unnecessary authority. | SCOPE-03 | FR-04 | story:VGLDQApqs |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hand lifecycle changes remain visible through trace, diagnostics, and native transport surfaces. | SCOPE-01 | NFR-02 | manual |
| SRS-NFR-02 | The migration preserves authored-workspace boundaries and existing operator-facing local execution behavior. | SCOPE-02 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
