---
# system-managed
id: VJZ9qJ9G6
status: done
created_at: 2026-05-13T21:35:44
updated_at: 2026-05-13T21:55:58
# authored
title: Document HTTP-Only Inference Decision
type: docs
operator-signal:
scope: VJZ034dF2/VJZ8Bws9Z
index: 3
started_at: 2026-05-13T21:53:00
submitted_at: 2026-05-13T21:55:56
completed_at: 2026-05-13T21:55:58
---

# Document HTTP-Only Inference Decision

## Summary

Update the owning architecture and configuration docs after the ADR is adopted.
This story makes the decision visible to operators without changing runtime
behavior beyond the documented compatibility policy.

## Acceptance Criteria

- [x] Architecture and configuration docs point to the HTTP-only inference ADR. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Docs stop presenting in-process model loading as the future-supported inference path. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Documentation uses `ollama:<model>` as the local HTTP inference form without naming a fixed default model. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
