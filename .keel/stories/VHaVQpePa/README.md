---
# system-managed
id: VHaVQpePa
status: done
created_at: 2026-04-22T22:10:00
updated_at: 2026-04-22T23:12:44
# authored
title: Carry External Provenance Through Transit Commands Events And Projections
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcsMav
index: 2
started_at: 2026-04-22T23:11:03
completed_at: 2026-04-22T23:12:44
---

# Carry External Provenance Through Transit Commands Events And Projections

## Summary

Thread explicit external provenance through the public Transit envelopes so
Paddles commands, lifecycle events, and projections carry the identity context
downstream consumers need without moving auth ownership into Paddles.

## Acceptance Criteria

- [x] Command, event, and projection envelopes carry explicit provenance for account, session, workspace, route, request identity, and workspace posture. [SRS-02/AC-01] <!-- verify: cargo test transit_provenance_envelopes_ -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Invalid or incomplete provenance is rejected explicitly by contract validation. [SRS-02/AC-02] <!-- verify: cargo test transit_contract_rejects_missing_provenance -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
