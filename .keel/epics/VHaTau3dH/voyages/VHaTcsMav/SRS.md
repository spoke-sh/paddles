# Versioned Hosted Transit Contract And Projection Surface - SRS

## Summary

Epic: VHaTau3dH
Goal: Define the stable versioned hosted Transit contract, provenance envelopes, and consumer-facing projection payloads that replace web endpoints as the canonical integration boundary.

## Scope

### In Scope

- [SCOPE-01] Versioned Transit command, event, and projection envelope definitions for hosted external integration
- [SCOPE-02] Stream-family and payload semantics for bootstrap, turn submission, progress, rebuild, completion/failure, and restore
- [SCOPE-03] Provenance envelope fields for account, session, workspace, route, request, and posture identity
- [SCOPE-04] Consumer-facing projection payload shape with transcript/detail metadata

### Out of Scope

- [SCOPE-05] Hosted cursor/materialization checkpoint mechanics themselves
- [SCOPE-06] Consumer frontend rendering implementation
- [SCOPE-07] HTTP debug/operator surface changes except where needed to preserve compatibility while Transit becomes canonical

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define a stable versioned Transit contract for bootstrap, turn submission, turn progress, projection rebuilds, completion/failure, and session restore. | SCOPE-01 | FR-03 | automated |
| SRS-02 | Every command, event, and projection envelope in the external Transit contract must carry explicit version markers and provenance fields for account, session, workspace, route, request identity, and workspace posture. | SCOPE-03 | FR-04 | automated |
| SRS-03 | The projection contract must expose a consumer-facing payload containing transcript rows, turn status, replay revision metadata, trace/manifold availability, and restore identity/session context. | SCOPE-04 | FR-05 | automated |
| SRS-04 | Transit must become the canonical integration boundary for hosted external flows; optional HTTP/operator surfaces may mirror or inspect the same state but must not define the contract. | SCOPE-01 | FR-09 | automated |
| SRS-05 | Documentation and contract tests must describe the stream families, payload invariants, and compatibility expectations for the versioned contract. | SCOPE-04 | FR-10 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The contract must remain backwards-disciplined enough that downstream consumers can use it without scraping incidental runtime or UI details. | SCOPE-01 | NFR-03 | automated |
| SRS-NFR-02 | Projection payloads must remain replay-derived views over authoritative Transit history rather than ad hoc web-session state. | SCOPE-03 | NFR-01 | automated |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
