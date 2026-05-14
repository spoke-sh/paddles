---
# system-managed
id: VJZ8KHZvT
status: backlog
created_at: 2026-05-13T21:29:43
updated_at: 2026-05-13T21:36:09
# authored
title: Resolve Final Rendering Through HTTP Model Clients
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8CYrLb
index: 2
---

# Resolve Final Rendering Through HTTP Model Clients

## Summary

Move final-rendering model construction to the HTTP model-client boundary. The
turn loop should receive a final-rendering client without paddles preparing a
local inference model.

## Acceptance Criteria

- [ ] A failing test is added first proving final-rendering runtime construction never receives local `ModelPaths`. [SRS-02/AC-01] <!-- verify: automated, SRS-02:start:end -->
- [ ] Final-rendering clients are built through HTTP provider configuration and capability negotiation. [SRS-02/AC-02] <!-- verify: automated, SRS-02:start:end -->
- [ ] HTTP provider tests for structured final answers, retries, and provider-specific schema behavior remain green. [SRS-02/AC-03] <!-- verify: automated, SRS-02:start:end -->
