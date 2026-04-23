# VOYAGE REPORT: Versioned Hosted Transit Contract And Projection Surface

## Voyage Metadata
- **ID:** VHaTcsMav
- **Epic:** VHaTau3dH
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Versioned Hosted Transit Envelopes And Stream Families
- **ID:** VHaVQDBoj
- **Status:** done

#### Summary
Define the versioned hosted Transit stream families and envelope layout so
external clients can bootstrap sessions and submit turns over Transit instead
of depending on the web transport as the canonical boundary.

#### Acceptance Criteria
- [x] The hosted Transit contract defines versioned envelopes for bootstrap, turn submission, progress, projection rebuild, completion/failure, and restore. [SRS-01/AC-01] <!-- verify: cargo test hosted_transit_contract_versions_ -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The hosted Transit stream families provide the command/event/projection layout the runtime will build on in later stories. [SRS-01/AC-02] <!-- verify: cargo test hosted_transit_stream_families_define_runtime_layout -- --nocapture, SRS-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHaVQDBoj/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHaVQDBoj/EVIDENCE/ac-2.log)

### Carry External Provenance Through Transit Commands Events And Projections
- **ID:** VHaVQpePa
- **Status:** done

#### Summary
Thread explicit external provenance through the public Transit envelopes so
Paddles commands, lifecycle events, and projections carry the identity context
downstream consumers need without moving auth ownership into Paddles.

#### Acceptance Criteria
- [x] Command, event, and projection envelopes carry explicit provenance for account, session, workspace, route, request identity, and workspace posture. [SRS-02/AC-01] <!-- verify: cargo test transit_provenance_envelopes_ -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Invalid or incomplete provenance is rejected explicitly by contract validation. [SRS-02/AC-02] <!-- verify: cargo test transit_contract_rejects_missing_provenance -- --nocapture, SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHaVQpePa/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHaVQpePa/EVIDENCE/ac-2.log)

### Publish Consumer-Facing Paddles Projection Payloads
- **ID:** VHaVRREzt
- **Status:** done

#### Summary
Publish the replay-derived projection payloads that downstream consumers can
use for transcript/detail rendering and deterministic restore, with Transit
rather than HTTP acting as the canonical projection surface.

#### Acceptance Criteria
- [x] The projection contract publishes transcript rows, turn status, replay revision metadata, and trace/manifold availability in a typed consumer-facing payload. [SRS-03/AC-01] <!-- verify: cargo test consumer_projection_payloads_include_transcript_status_and_revision_metadata -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Projection tests prove the payload remains replay-derived and Transit-canonical rather than reconstructed from web-session state. [SRS-04/AC-02] <!-- verify: cargo test transit_projection_payloads_remain_replay_derived -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
- [x] Projection payloads remain replay-derived views over authoritative Transit history rather than ad hoc web-session state. [SRS-NFR-02/AC-03] <!-- verify: cargo test consumer_projection_payloads_remain_replay_derived_views -- --nocapture, SRS-NFR-02:start:end, proof: ac-3.log-->
- [x] The versioned stream families, payload invariants, and compatibility expectations are documented with the published projection contract. [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end, proof: ac-4.log-->
- [x] Contract tests cover the public envelopes and reject unsupported or malformed versions without requiring UI scraping. [SRS-NFR-01/AC-05] <!-- verify: cargo test hosted_transit_contract_rejects_unsupported_versions -- --nocapture, SRS-NFR-01:start:end, proof: ac-5.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHaVRREzt/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHaVRREzt/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VHaVRREzt/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VHaVRREzt/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/VHaVRREzt/EVIDENCE/ac-5.log)


