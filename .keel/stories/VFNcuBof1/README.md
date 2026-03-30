---
# system-managed
id: VFNcuBof1
status: backlog
created_at: 2026-03-30T12:20:23
updated_at: 2026-03-30T12:24:40
# authored
title: Colored Delta Text In Transcript Rows
type: feat
operator-signal:
scope: VFNccFj7d/VFNcoxjU3
index: 4
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

- [ ] Delta text renders in a dim style when classified as fast [SRS-10/AC-01] <!-- verify: manual, SRS-10:start:end -->
- [ ] Delta text renders in default style when classified as normal [SRS-10/AC-02] <!-- verify: manual, SRS-10:start:end -->
- [ ] Delta text renders in a warm highlight when classified as slow [SRS-10/AC-03] <!-- verify: manual, SRS-10:start:end -->
- [ ] Palette includes pace styles for both light and dark themes [SRS-11/AC-04] <!-- verify: test, SRS-11:start:end -->
