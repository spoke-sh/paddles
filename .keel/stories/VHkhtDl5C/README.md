---
# system-managed
id: VHkhtDl5C
status: icebox
created_at: 2026-04-24T16:02:13
updated_at: 2026-04-24T16:06:26
# authored
title: Inherit Governance And Budgets Into Workers
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgMxksP
index: 2
---

# Inherit Governance And Budgets Into Workers

## Summary

Inherit governance, execution policy, capability posture, and budget limits into worker contexts so delegation cannot widen authority.

## Acceptance Criteria

- [ ] Worker contexts inherit parent governance, execution policy, capability posture, and budget limits. [SRS-02/AC-01] <!-- verify: cargo test worker_inherits_governance -- --nocapture, SRS-02:start:end -->
- [ ] Worker execution cannot use capabilities unavailable to the parent turn. [SRS-NFR-01/AC-01] <!-- verify: cargo test worker_authority_bounds -- --nocapture, SRS-NFR-01:start:end -->
