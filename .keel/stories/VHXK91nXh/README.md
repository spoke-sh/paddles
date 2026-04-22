---
# system-managed
id: VHXK91nXh
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:34:17
# authored
title: Preserve Moonshot Reasoning Continuity Across Tool Turns
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipEBj
index: 3
started_at: 2026-04-22T09:28:13
completed_at: 2026-04-22T09:34:17
---

# Preserve Moonshot Reasoning Continuity Across Tool Turns

## Summary

Use the new substrate to carry Moonshot/Kimi reasoning continuity through
recursive tool/result turns so the provider can preserve its native thinking
state without leaking raw reasoning into canonical turn output.

## Acceptance Criteria

- [x] Moonshot/Kimi preserves required provider-native reasoning continuity across tool/result turns. [SRS-03/AC-01] <!-- verify: cargo test moonshot_prompt_envelope_replays_reasoning_state_before_tool_results -- --nocapture, SRS-03:start:end -->
- [x] Canonical transcript/render output remains free of raw Moonshot reasoning artifacts. [SRS-04/AC-02] <!-- verify: cargo test moonshot_prompt_envelope_captures_reasoning_tool_state_without_exposing_it_as_content -- --nocapture, SRS-04:start:end -->
