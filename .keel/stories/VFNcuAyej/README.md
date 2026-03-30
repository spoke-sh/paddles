---
# system-managed
id: VFNcuAyej
status: backlog
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:24:40
# authored
title: Percentile Based Pace Classification
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 3
---

# Percentile Based Pace Classification

## Summary

Add a `classify(key, delta) -> Pace` method to the reservoir that returns Fast, Normal, or Slow based on historical percentiles.

Classification rules:
- delta < p50 → Fast (step completed quicker than typical)
- p50 ≤ delta ≤ p85 → Normal
- delta > p85 → Slow (step took notably longer than typical)
- Insufficient history (< 5 samples) → Normal (don't guess)

Expose a `Pace` enum that the rendering layer can match on.

## Acceptance Criteria

- [ ] classify returns Normal when fewer than 5 samples exist for the key [SRS-09/AC-01] <!-- verify: test, SRS-09:start:end -->
- [ ] classify returns Fast for delta below p50 [SRS-08/AC-02] <!-- verify: test, SRS-08:start:end -->
- [ ] classify returns Normal for delta between p50 and p85 [SRS-08/AC-03] <!-- verify: test, SRS-08:start:end -->
- [ ] classify returns Slow for delta above p85 [SRS-08/AC-04] <!-- verify: test, SRS-08:start:end -->
