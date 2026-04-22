---
# system-managed
id: VHXK91nXh
status: backlog
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:14:06
# authored
title: Preserve Moonshot Reasoning Continuity Across Tool Turns
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipEBj
index: 3
---

# Preserve Moonshot Reasoning Continuity Across Tool Turns

## Summary

Use the new substrate to carry Moonshot/Kimi reasoning continuity through
recursive tool/result turns so the provider can preserve its native thinking
state without leaking raw reasoning into canonical turn output.

## Acceptance Criteria

- [ ] Moonshot/Kimi preserves required provider-native reasoning continuity across tool/result turns. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Canonical transcript/render output remains free of raw Moonshot reasoning artifacts. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
