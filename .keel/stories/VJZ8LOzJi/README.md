---
# system-managed
id: VJZ8LOzJi
status: icebox
created_at: 2026-05-13T21:29:47
updated_at: 2026-05-13T21:38:08
# authored
title: Introduce Turn Runtime Preference Schema
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 1
---

# Introduce Turn Runtime Preference Schema

## Summary

Introduce the canonical turn-runtime preference schema. New code should describe
model clients and turn phases directly instead of persisting planner,
synthesizer, gatherer, or runtime-lane settings.

## Acceptance Criteria

- [ ] Tests define the new preference shape using action-selection, final-rendering, retrieval, model-client, and turn-runtime terminology. [SRS-01/AC-01] <!-- verify: automated, SRS-01:start:end -->
- [ ] New preference writes do not emit planner, synthesizer, gatherer, or lane-shaped field names. [SRS-01/AC-02] <!-- verify: automated, SRS-01:start:end -->
- [ ] Runtime construction consumes normalized turn-runtime preferences. [SRS-01/AC-03] <!-- verify: automated, SRS-01:start:end -->
