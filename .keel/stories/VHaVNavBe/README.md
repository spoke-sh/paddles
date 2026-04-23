---
# system-managed
id: VHaVNavBe
status: in-progress
created_at: 2026-04-22T22:09:48
updated_at: 2026-04-22T22:12:31
# authored
title: Define Hosted Transit Authority Config And Runtime Contract
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcrsZq
index: 1
started_at: 2026-04-22T22:12:31
---

# Define Hosted Transit Authority Config And Runtime Contract

## Summary

Define the hosted Transit authority/config seam for deployed service mode so the
runtime can distinguish authoritative hosted operation from explicit local/dev
fallbacks before recorder and resume implementation begins.

## Acceptance Criteria

- [ ] Runtime configuration can select hosted Transit authority mode explicitly, including Transit endpoint, namespace, and service identity requirements. [SRS-02/AC-01] <!-- verify: cargo test hosted_transit_authority_config_ -- --nocapture, SRS-02:start:end -->
- [ ] Hosted service-mode config rejects implicit fallback to embedded local storage when required hosted fields are missing. [SRS-02/AC-02] <!-- verify: cargo test hosted_service_mode_rejects_implicit_local_fallback -- --nocapture, SRS-02:start:end -->
- [ ] Local/dev fallback modes remain explicit and separate from hosted first-party deployment semantics. [SRS-03/AC-03] <!-- verify: cargo test recorder_authority_modes_require_explicit_selection -- --nocapture, SRS-03:start:end -->
