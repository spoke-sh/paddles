---
# system-managed
id: VFJ5wTofV
status: backlog
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T17:47:05
# authored
title: Shift Retrieval Selection And Evidence Prioritization Toward Model Judgement
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 3
---

# Shift Retrieval Selection And Evidence Prioritization Toward Model Judgement

## Summary

Move retrieval-selection and evidence-prioritization choices that represent
reasoning into constrained model judgement so recursive turns stop depending on
static lexical defaults and hardcoded source-priority rankings where the model
should decide.

## Acceptance Criteria

- [ ] Retrieval query/mode selection on recursive turns no longer depends on hardcoded reasoning heuristics where the model can decide the better move. [SRS-04/AC-01] <!-- verify: automated, SRS-04:start:end -->
- [ ] Evidence prioritization used for recursive reasoning/synthesis is revised so reasoning-heavy ranking is not encoded as static controller policy. [SRS-04/AC-02] <!-- verify: automated/manual, SRS-04:start:end -->
- [ ] Controller-owned safety constraints for execution and resource bounds remain unchanged. [SRS-NFR-01/AC-03] <!-- verify: automated, SRS-NFR-01:start:end -->
