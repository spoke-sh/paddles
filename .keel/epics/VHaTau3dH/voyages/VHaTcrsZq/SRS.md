# Hosted Transit Authority And Service Runtime Mode - SRS

## Summary

Epic: VHaTau3dH
Goal: Make hosted Transit the authoritative runtime path for deployed Paddles while preserving embedded/local recorders only as explicit local fallback or dev mode.

## Scope

### In Scope

- [SCOPE-01] Hosted Transit authority mode configuration and negotiation for deployed service runtime
- [SCOPE-02] `transit-client` backed recorder/service seams for authoritative append/read operations
- [SCOPE-03] Non-interactive service-mode bootstrap, readiness, and failure reporting
- [SCOPE-04] Explicit local/dev fallback modes for embedded local `transit-core` and in-memory recorders
- [SCOPE-05] Documentation and contract tests that distinguish hosted authority mode from debug/operator HTTP surfaces

### Out of Scope

- [SCOPE-06] Versioning of the full external Transit envelope schema beyond the runtime hooks needed to host the service
- [SCOPE-07] Consumer payload shaping beyond the runtime metadata needed to emit projections
- [SCOPE-08] Downstream deployment automation or auth ownership changes

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The runtime must expose a hosted Transit authority mode that binds deployed recorder, replay, and session bootstrap behavior to hosted Transit through `transit-client`. | SCOPE-01 | FR-01 | automated |
| SRS-02 | Hosted Transit authority mode must be configurable with explicit Transit endpoint, namespace, service identity, and authority-mode selection rather than silently falling back to embedded local storage. | SCOPE-01 | FR-08 | automated |
| SRS-03 | Embedded local `transit-core` and in-memory recorder paths must remain available only as explicit local/dev fallback modes, and the deployed first-party path must not require them. | SCOPE-03 | FR-02 | automated |
| SRS-04 | Service mode must expose non-interactive readiness and failure state that operators can observe without attaching the TUI or relying on the web UI. | SCOPE-03 | NFR-04 | automated |
| SRS-05 | Optional HTTP UI, debug, and operator surfaces must remain decoupled from the hosted service authority contract so they can be disabled without breaking the primary hosted Transit integration path. | SCOPE-05 | FR-09 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Hosted service mode must maintain a single recorder authority and must not open embedded local Transit storage as a second authority for the same workload. | SCOPE-01 | NFR-02 | automated |
| SRS-NFR-02 | Hosted service-mode behavior and fallback behavior must be documented clearly enough that operators can tell which authority path is active. | SCOPE-05 | FR-10 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
