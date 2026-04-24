---
# system-managed
id: VHkhtzhw7
status: icebox
created_at: 2026-04-24T16:02:16
updated_at: 2026-04-24T16:06:30
# authored
title: Integrate Worker Evidence In Parent Trace
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgMxksP
index: 3
---

# Integrate Worker Evidence In Parent Trace

## Summary

Merge worker findings, artifacts, and edit proposals back into parent-loop evidence with explicit integration status.

## Acceptance Criteria

- [ ] Worker outputs become parent-loop evidence with accepted, rejected, or needs-integration status. [SRS-03/AC-01] <!-- verify: cargo test worker_evidence_integration -- --nocapture, SRS-03:start:end -->
- [ ] Parent integration owns conflict handling and does not silently apply unmanaged worker changes. [SRS-03/AC-02] <!-- verify: cargo test worker_integration_conflicts -- --nocapture, SRS-03:start:end -->
