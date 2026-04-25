---
# system-managed
id: VHkhxz6zK
status: done
created_at: 2026-04-24T16:02:31
updated_at: 2026-04-24T19:09:37
# authored
title: Add Provider Model Registry Posture
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgPmlyS
index: 2
started_at: 2026-04-24T19:07:27
completed_at: 2026-04-24T19:09:37
---

# Add Provider Model Registry Posture

## Summary

Add provider and model registry posture that distinguishes configured, discovered, unavailable, and deprecated model entries without forcing network discovery.

## Acceptance Criteria

- [x] Provider/model registry state reports configured, discovered, unavailable, and deprecated entries. [SRS-02/AC-01] <!-- verify: cargo test provider_registry_posture -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Default local-first mode does not require network discovery to build provider posture. [SRS-NFR-02/AC-01] <!-- verify: cargo test provider_registry_offline -- --nocapture, SRS-NFR-02:start:end, proof: ac-2.log-->
