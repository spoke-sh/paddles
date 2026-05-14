---
# system-managed
id: VJZ1Krd85
status: backlog
created_at: 2026-05-13T21:01:57
updated_at: 2026-05-13T21:03:32
# authored
title: Inventory HTTP Model Client Seams
type: chore
operator-signal:
scope: VJZ0tpZQJ/VJZ14yp0U
index: 2
---

# Inventory HTTP Model Client Seams

## Summary

Map the existing HTTP provider and capability-negotiation seams that can become
the sole model inference boundary. The output should show how local models can
remain local-first by running behind HTTP services rather than being loaded
inside paddles.

## Acceptance Criteria

- [ ] Inventory lists HTTP provider/model-client files, provider capability surfaces, planner/synthesizer factory seams, and provider URL/auth configuration involved in inference transport. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Inventory explains how local HTTP-backed providers such as Ollama fit the target boundary without paddles-owned model loading. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] Inventory identifies test anchors that prove HTTP-backed planner and answer paths still receive the correct capability and action-schema contracts. [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end -->
