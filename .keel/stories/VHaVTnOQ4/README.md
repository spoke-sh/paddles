---
# system-managed
id: VHaVTnOQ4
status: done
created_at: 2026-04-22T22:10:12
updated_at: 2026-04-22T23:44:10
# authored
title: Verify Hosted Restart Resume Fidelity Against Authoritative Transit History
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcrQZr
index: 3
started_at: 2026-04-22T23:41:08
completed_at: 2026-04-22T23:44:10
---

# Verify Hosted Restart Resume Fidelity Against Authoritative Transit History

## Summary

Verify that hosted restart/resume preserves replay-derived truth by exercising
no-loss, no-duplication, and full-replay fallback scenarios against
authoritative Transit history.

## Acceptance Criteria

- [x] Deterministic restart scenarios prove hosted cursor/checkpoint resume does not lose, reorder, or duplicate work. [SRS-03/AC-01] <!-- verify: cargo test hosted_restart_resume_fidelity_ -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Full replay remains available as a correctness baseline when hosted resume state is missing or invalid. [SRS-04/AC-02] <!-- verify: cargo test hosted_resume_falls_back_to_authoritative_replay -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
- [x] Verification artifacts show that hosted resume state does not become a second source of truth. [SRS-NFR-01/AC-03] <!-- verify: cargo test hosted_resume_preserves_replay_derived_truth -- --nocapture, SRS-NFR-01:start:end, proof: ac-3.log-->
- [x] Resume correctness is reproducible through deterministic restart scenarios that cover no-loss and no-duplication behavior. [SRS-NFR-02/AC-04] <!-- verify: cargo test hosted_restart_resume_is_deterministic -- --nocapture, SRS-NFR-02:start:end, proof: ac-4.log-->
- [x] Hosted projections and diagnostics expose enough replay revision metadata to explain the resumed cursor and checkpoint state. [SRS-05/AC-05] <!-- verify: cargo test hosted_projection_resume_metadata_is_exposed -- --nocapture, SRS-05:start:end, proof: ac-5.log-->
