---
# system-managed
id: VHaVP94kM
status: backlog
created_at: 2026-04-22T22:09:54
updated_at: 2026-04-22T22:12:06
# authored
title: Implement Hosted Transit Recorder And Service Bootstrap
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcrsZq
index: 2
---

# Implement Hosted Transit Recorder And Service Bootstrap

## Summary

Implement the hosted Transit-backed recorder/bootstrap path so deployed Paddles
can bind core recorder and replay seams to `transit-client` without requiring
embedded local `transit-core`.

## Acceptance Criteria

- [ ] Hosted authority mode binds recorder and replay operations to a hosted Transit-backed implementation. [SRS-01/AC-01] <!-- verify: cargo test hosted_transit_trace_store_ -- --nocapture, SRS-01:start:end -->
- [ ] Hosted service bootstrap can start against hosted Transit without embedded local Transit storage when hosted authority mode is selected. [SRS-01/AC-02] <!-- verify: cargo test hosted_service_mode_does_not_require_embedded_transit_core -- --nocapture, SRS-01:start:end -->
- [ ] Hosted authority mode maintains a single recorder authority and does not reopen embedded local Transit storage for the same workload. [SRS-NFR-01/AC-03] <!-- verify: cargo test hosted_authority_mode_preserves_single_recorder_authority -- --nocapture, SRS-NFR-01:start:end -->
