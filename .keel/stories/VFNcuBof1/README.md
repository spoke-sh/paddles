---
# system-managed
id: VFNcuBof1
status: done
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:40:01
# authored
title: Colored Delta Text In Transcript Rows
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 4
started_at: 2026-03-30T12:37:18
submitted_at: 2026-03-30T12:39:56
completed_at: 2026-03-30T12:40:01
---

# Colored Delta Text In Transcript Rows

## Summary

Split the timing label rendering so the delta portion `(+Xs)` is styled by pace classification. The elapsed portion and header remain in their existing styles.

Color treatment:
- Fast → dim/muted (the step is uninteresting, recede it)
- Normal → default body style (no change from today)
- Slow → warm highlight color (draw the eye to the bottleneck)

Add `pace_fast`, `pace_normal`, `pace_slow` styles to the Palette for both light and dark themes.

## Acceptance Criteria

- [x] Delta text renders in a dim style when classified as fast [SRS-10/AC-01] <!-- verify: manual, SRS-10:start:end, proof: ac-1.log-->
- [x] Delta text renders in default style when classified as normal [SRS-10/AC-02] <!-- verify: manual, SRS-10:start:end, proof: ac-2.log-->
- [x] Delta text renders in a warm highlight when classified as slow [SRS-10/AC-03] <!-- verify: manual, SRS-10:start:end, proof: ac-3.log-->
- [x] Palette includes pace styles for both light and dark themes [SRS-11/AC-04] <!-- verify: manual, SRS-11:start:end, proof: ac-4.log-->
