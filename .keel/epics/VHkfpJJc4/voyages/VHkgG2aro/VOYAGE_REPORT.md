# VOYAGE REPORT: Implement Governed External Capability Broker

## Voyage Metadata
- **ID:** VHkgG2aro
- **Epic:** VHkfpJJc4
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define External Capability Broker Port And Catalog
- **ID:** VHkhj5rdT
- **Status:** done

#### Summary
Define the runtime broker port and capability catalog needed to replace the noop external capability broker without forcing network access.

#### Acceptance Criteria
- [x] A broker registry exposes declared external capability availability through a domain/application boundary. [SRS-01/AC-01] <!-- verify: cargo test external_capability_broker -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The default catalog remains unavailable unless local configuration enables a capability. [SRS-NFR-01/AC-01] <!-- verify: cargo test external_capability_default_posture -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHkhj5rdT/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHkhj5rdT/EVIDENCE/ac-2.log)

### Govern External Capability Invocation Results
- **ID:** VHkhk8Xmt
- **Status:** done

#### Summary
Route external capability invocations through governance and return typed results for allowed, denied, unavailable, degraded, and malformed outcomes.

#### Acceptance Criteria
- [x] External capability calls consult governance before executing side effects. [SRS-02/AC-01] <!-- verify: cargo test external_capability_governance -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Denied and degraded external calls return typed evidence rather than opaque errors. [SRS-02/AC-02] <!-- verify: cargo test external_capability_result_states -- --nocapture, SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHkhk8Xmt/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHkhk8Xmt/EVIDENCE/ac-2.log)

### Feed External Results Into Recursive Evidence
- **ID:** VHkhkxETY
- **Status:** done

#### Summary
Attach external capability results to recursive evidence and projection events so the planner and operator see the same provenance-bearing runtime facts.

#### Acceptance Criteria
- [x] External capability results append structured evidence to the planner loop. [SRS-03/AC-01] <!-- verify: cargo test external_capability_evidence -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Projection events expose external capability provenance and degraded states. [SRS-NFR-02/AC-01] <!-- verify: cargo test external_capability_projection -- --nocapture, SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHkhkxETY/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHkhkxETY/EVIDENCE/ac-2.log)


