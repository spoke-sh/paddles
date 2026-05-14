---
# system-managed
id: VJZ8IcONz
status: done
created_at: 2026-05-13T21:29:36
updated_at: 2026-05-13T21:43:43
# authored
title: Adopt HTTP-Only Model Inference ADR
type: chore
operator-signal:
scope: VJZ034dF2/VJZ8Bws9Z
index: 1
started_at: 2026-05-13T21:41:34
submitted_at: 2026-05-13T21:43:38
completed_at: 2026-05-13T21:43:43
---

# Adopt HTTP-Only Model Inference ADR

## Summary

Adopt the ADR that makes HTTP model clients the only supported inference
boundary for action selection and final rendering. The story should also add
guardrails that keep docs and future code aligned with the decision.

## Acceptance Criteria

- [x] ADR states paddles no longer loads inference models in-process for action selection or final rendering. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] ADR states local-first inference is supported through HTTP model services and uses `ollama:<model>` as the canonical local provider form. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] Architecture/configuration docs reference the ADR and stop presenting in-process local model loading as the future inference path. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end, proof: ac-3.log-->
