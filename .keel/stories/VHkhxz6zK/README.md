---
# system-managed
id: VHkhxz6zK
status: icebox
created_at: 2026-04-24T16:02:31
updated_at: 2026-04-24T16:06:47
# authored
title: Add Provider Model Registry Posture
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgPmlyS
index: 2
---

# Add Provider Model Registry Posture

## Summary

Add provider and model registry posture that distinguishes configured, discovered, unavailable, and deprecated model entries without forcing network discovery.

## Acceptance Criteria

- [ ] Provider/model registry state reports configured, discovered, unavailable, and deprecated entries. [SRS-02/AC-01] <!-- verify: cargo test provider_registry_posture -- --nocapture, SRS-02:start:end -->
- [ ] Default local-first mode does not require network discovery to build provider posture. [SRS-NFR-02/AC-01] <!-- verify: cargo test provider_registry_offline -- --nocapture, SRS-NFR-02:start:end -->
