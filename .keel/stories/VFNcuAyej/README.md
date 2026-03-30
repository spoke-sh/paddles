---
# system-managed
id: VFNcuAyej
status: done
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:40:01
# authored
title: Percentile Based Pace Classification
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 3
started_at: 2026-03-30T12:37:18
submitted_at: 2026-03-30T12:39:56
completed_at: 2026-03-30T12:40:01
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

- [x] classify returns Normal when fewer than 5 samples exist for the key [SRS-09/AC-01] <!-- verify: manual, SRS-09:start:end, proof: ac-1.log-->
- [x] classify returns Fast for delta below p50 [SRS-08/AC-02] <!-- verify: manual, SRS-08:start:end, proof: ac-2.log-->
- [x] classify returns Normal for delta between p50 and p85 [SRS-08/AC-03] <!-- verify: manual, SRS-08:start:end, proof: ac-3.log-->
- [x] classify returns Slow for delta above p85 [SRS-08/AC-04] <!-- verify: manual, SRS-08:start:end, proof: ac-4.log-->
