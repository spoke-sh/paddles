---
# system-managed
id: VFURsk7Xz
status: done
created_at: 2026-03-31T16:20:30
updated_at: 2026-03-31T16:34:39
# authored
title: Add Refinement Cooldown And Oscillation Prevention
type: feat
operator-signal:
scope: VFUNJz9zT/VFURjeU0t
index: 4
started_at: 2026-03-31T16:31:16
submitted_at: 2026-03-31T16:34:31
completed_at: 2026-03-31T16:34:39
---

# Add Refinement Cooldown And Oscillation Prevention

## Summary

Add cooldown windows and oscillation guardrails for refinements to prevent repeated context churn and unstable planner behavior.

## Acceptance Criteria

- [x] Apply cooldown and oscillation-avoidance checks so repeated refinements are bounded, deterministic, and skip when policy stability would be degraded. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
