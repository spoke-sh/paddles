# Deliver WebSocket And Transit Transports - SRS

## Summary

Epic: VGKnsYg1z
Goal: Add bidirectional WebSocket and Transit-native transport adapters using the same shared lifecycle, visibility, and failure semantics as the HTTP-facing transports.

## Scope

### In Scope

- [SCOPE-01] Implement a native WebSocket transport adapter for bidirectional session-oriented communication on the shared transport contract.
- [SCOPE-02] Implement a native Transit transport adapter that exposes structured Transit-native exchange semantics through the same shared lifecycle and diagnostics model.
- [SCOPE-03] Verify the bidirectional transport adapters with diagnostics, docs, and failure handling that keep WebSocket and Transit understandable to operators.

### Out of Scope

- [SCOPE-04] Reworking the already-landed shared transport substrate or HTTP/SSE adapter behavior except where required for compatibility.
- [SCOPE-05] Adding third-party brokers, hosted relays, or unrelated transport protocols.
- [SCOPE-06] Changing planner/runtime product defaults beyond what is necessary to configure and observe the new native transports.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Add a WebSocket transport adapter that supports bidirectional session communication while reporting readiness, negotiated capabilities, and failures through the shared transport diagnostics model. | SCOPE-01 | FR-04 | tests |
| SRS-02 | Add a Transit-native transport adapter that exchanges structured Transit payloads under the shared transport lifecycle, auth, and diagnostics contract. | SCOPE-02 | FR-05 | tests |
| SRS-03 | Document and verify bidirectional transport flows, diagnostics, and recovery behavior so operators can distinguish WebSocket and Transit readiness/failure clearly. | SCOPE-03 | FR-06 | tests + docs |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Keep WebSocket and Transit adapters aligned to the shared transport semantics so new bidirectional transports do not reintroduce protocol-specific operator drift. | SCOPE-01, SCOPE-02 | NFR-01 | review + tests |
| SRS-NFR-02 | Fail closed with clear diagnostics when transport negotiation, bind, or session setup cannot complete, rather than leaving partially ready bidirectional channels. | SCOPE-03 | NFR-02 | tests |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
