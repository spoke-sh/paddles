---
# system-managed
id: VHkhkxETY
status: done
created_at: 2026-04-24T16:01:41
updated_at: 2026-04-24T17:50:49
# authored
title: Feed External Results Into Recursive Evidence
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgG2aro
index: 3
started_at: 2026-04-24T17:49:23
completed_at: 2026-04-24T17:50:49
---

# Feed External Results Into Recursive Evidence

## Summary

Attach external capability results to recursive evidence and projection events so the planner and operator see the same provenance-bearing runtime facts.

## Acceptance Criteria

- [x] External capability results append structured evidence to the planner loop. [SRS-03/AC-01] <!-- verify: cargo test external_capability_evidence -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Projection events expose external capability provenance and degraded states. [SRS-NFR-02/AC-01] <!-- verify: cargo test external_capability_projection -- --nocapture, SRS-NFR-02:start:end, proof: ac-2.log-->
