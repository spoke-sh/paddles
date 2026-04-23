---
# system-managed
id: VHaVPgBE0
status: done
created_at: 2026-04-22T22:09:56
updated_at: 2026-04-22T23:05:53
# authored
title: Add Hosted Service Readiness And Operator Surface Boundaries
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcrsZq
index: 3
started_at: 2026-04-22T23:02:08
submitted_at: 2026-04-22T23:05:47
completed_at: 2026-04-22T23:05:53
---

# Add Hosted Service Readiness And Operator Surface Boundaries

## Summary

Add the non-interactive service supervision and operator-surface boundaries for
hosted mode so readiness/failure is first-class and optional HTTP surfaces stop
defining the primary deployment contract.

## Acceptance Criteria

- [x] Hosted service mode exposes readiness and failure state without requiring the TUI or web UI to be attached. [SRS-04/AC-01] <!-- verify: cargo test hosted_service_runtime_reports_readiness_and_failure_state -- --nocapture, SRS-04:start:end, proof: ac-1.log-->
- [x] Optional HTTP/operator surfaces can be disabled without breaking the primary hosted Transit service path. [SRS-05/AC-02] <!-- verify: cargo test hosted_service_mode_keeps_operator_surfaces_optional -- --nocapture, SRS-05:start:end, proof: ac-2.log-->
- [x] Hosted service-mode and fallback behavior are documented clearly enough that operators can tell which authority path is active. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
