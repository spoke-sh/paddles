# VOYAGE REPORT: Expose Operator Surfaces And Provider Registry

## Voyage Metadata
- **ID:** VHkgPmlyS
- **Epic:** VHkfpJJc4
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Surface Runtime Posture In Operator Projections
- **ID:** VHkhxGnGQ
- **Status:** done

#### Summary
Surface capability posture, governance decisions, diagnostics, worker evidence, and eval outcomes through existing operator projections.

#### Acceptance Criteria
- [x] CLI, TUI, or web projections expose new runtime posture events without inventing controller-authored plans. [SRS-01/AC-01] <!-- verify: cargo test runtime_posture_projection -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Projection snapshots include governance, diagnostics, provenance, worker, and eval outcome fields where present. [SRS-NFR-01/AC-01] <!-- verify: cargo test runtime_posture_projection -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHkhxGnGQ/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHkhxGnGQ/EVIDENCE/ac-2.log)

### Add Provider Model Registry Posture
- **ID:** VHkhxz6zK
- **Status:** done

#### Summary
Add provider and model registry posture that distinguishes configured, discovered, unavailable, and deprecated model entries without forcing network discovery.

#### Acceptance Criteria
- [x] Provider/model registry state reports configured, discovered, unavailable, and deprecated entries. [SRS-02/AC-01] <!-- verify: cargo test provider_registry_posture -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Default local-first mode does not require network discovery to build provider posture. [SRS-NFR-02/AC-01] <!-- verify: cargo test provider_registry_offline -- --nocapture, SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHkhxz6zK/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHkhxz6zK/EVIDENCE/ac-2.log)

### Document Harness Capability Configuration
- **ID:** VHkhz3Y8v
- **Status:** done

#### Summary
Document capability configuration, eval usage, and local-first boundaries for operators adopting the upgraded harness.

#### Acceptance Criteria
- [x] Operator docs explain configuring external capabilities, execution policy, evals, and provider posture. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Runtime entrypoint smoke checks confirm the documented surfaces expose the new harness posture consistently. [SRS-04/AC-01] <!-- verify: cargo test runtime_entrypoint_smoke -- --nocapture, SRS-04:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHkhz3Y8v/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHkhz3Y8v/EVIDENCE/ac-2.log)


