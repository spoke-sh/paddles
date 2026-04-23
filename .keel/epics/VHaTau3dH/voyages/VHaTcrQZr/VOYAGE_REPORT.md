# VOYAGE REPORT: Hosted Cursor Resume And Projection Rebuild Semantics

## Voyage Metadata
- **ID:** VHaTcrQZr
- **Epic:** VHaTau3dH
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Add Hosted Consumer Cursor Resume For Session Readers
- **ID:** VHaVRxBd9
- **Status:** done

#### Summary
Add hosted consumer cursor ownership for session and lifecycle readers so
service restart can resume from authoritative hosted positions instead of
replaying from zero by default.

#### Acceptance Criteria
- [x] Session and lifecycle consumers persist hosted cursor positions and resume from them on restart. [SRS-01/AC-01] <!-- verify: cargo test hosted_session_cursor_resume_ -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Session readers expose resumed hosted cursor identity and position metadata for the running service instance. [SRS-01/AC-02] <!-- verify: cargo test hosted_session_cursor_resume_metadata_is_exposed -- --nocapture, SRS-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHaVRxBd9/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHaVRxBd9/EVIDENCE/ac-2.log)

### Add Hosted Materialization Checkpoints For Projection Rebuilds
- **ID:** VHaVTGut5
- **Status:** done

#### Summary
Add hosted materialization checkpoint/resume support for replay-derived
projections so projection rebuilds can restart efficiently without drifting from
authoritative Transit history.

#### Acceptance Criteria
- [x] Projection reducers persist and resume hosted materialization checkpoints or equivalent hosted resume tokens. [SRS-02/AC-01] <!-- verify: cargo test hosted_projection_materialization_checkpoints_ -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Hosted projection materializers can resume from persisted checkpoint state without requiring a local-only checkpoint store. [SRS-02/AC-02] <!-- verify: cargo test hosted_projection_materializers_resume_without_local_checkpoint_store -- --nocapture, SRS-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHaVTGut5/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHaVTGut5/EVIDENCE/ac-2.log)

### Verify Hosted Restart Resume Fidelity Against Authoritative Transit History
- **ID:** VHaVTnOQ4
- **Status:** done

#### Summary
Verify that hosted restart/resume preserves replay-derived truth by exercising
no-loss, no-duplication, and full-replay fallback scenarios against
authoritative Transit history.

#### Acceptance Criteria
- [x] Deterministic restart scenarios prove hosted cursor/checkpoint resume does not lose, reorder, or duplicate work. [SRS-03/AC-01] <!-- verify: cargo test hosted_restart_resume_fidelity_ -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Full replay remains available as a correctness baseline when hosted resume state is missing or invalid. [SRS-04/AC-02] <!-- verify: cargo test hosted_resume_falls_back_to_authoritative_replay -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
- [x] Verification artifacts show that hosted resume state does not become a second source of truth. [SRS-NFR-01/AC-03] <!-- verify: cargo test hosted_resume_preserves_replay_derived_truth -- --nocapture, SRS-NFR-01:start:end, proof: ac-3.log-->
- [x] Resume correctness is reproducible through deterministic restart scenarios that cover no-loss and no-duplication behavior. [SRS-NFR-02/AC-04] <!-- verify: cargo test hosted_restart_resume_is_deterministic -- --nocapture, SRS-NFR-02:start:end, proof: ac-4.log-->
- [x] Hosted projections and diagnostics expose enough replay revision metadata to explain the resumed cursor and checkpoint state. [SRS-05/AC-05] <!-- verify: cargo test hosted_projection_resume_metadata_is_exposed -- --nocapture, SRS-05:start:end, proof: ac-5.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VHaVTnOQ4/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VHaVTnOQ4/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VHaVTnOQ4/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VHaVTnOQ4/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/VHaVTnOQ4/EVIDENCE/ac-5.log)


