---
# system-managed
id: VFKN7NBCT
status: icebox
created_at: 2026-03-29T22:58:52
updated_at: 2026-03-29T22:58:52
# authored
title: Anthropic Claude HTTP Adapter
type: feat
operator-signal:
scope: VFKMmuJFY/VFKN5dR0L
index: 1
---

# Anthropic Claude HTTP Adapter

## Summary

Deliver a reqwest-based Anthropic messages API adapter that implements SynthesizerEngine and RecursivePlanner with correct API headers and top-level system prompt.

## Acceptance Criteria

- [x] Anthropic adapter implements SynthesizerEngine <!-- verify: manual, SRS-01:AC-01 -->
- [x] System prompt uses top-level system parameter <!-- verify: manual, SRS-02:AC-02 -->
- [x] Planner JSON parsing works <!-- verify: manual, SRS-03:AC-03 -->
- [x] Correct API headers <!-- verify: manual, SRS-NFR-01:AC-04 -->
