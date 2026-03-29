---
# system-managed
id: VFH4DWu9K
status: icebox
created_at: 2026-03-29T09:24:58
updated_at: 2026-03-29T09:24:58
# authored
title: Refactor Planner State And Turn Events Into Recorder-Ready Structured Traces
type: feat
operator-signal:
scope: VFH4BXH4F/VFH4CCJ4d
index: 2
---

# Refactor Planner State And Turn Events Into Recorder-Ready Structured Traces

## Summary

Refactor planner loop state and turn execution state so recorder-ready lineage
and branch structure exists independently of transcript rendering or ad hoc
string summaries.

## Acceptance Criteria

- [ ] Planner loop state preserves structured branch and lineage data instead of string-only pending branches. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Runtime trace projection derives durable trace entities before transcript rendering formats them for the UI. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] Operator-facing transcript behavior remains concise even though the underlying durable trace data is richer. [SRS-NFR-03/AC-03] <!-- verify: manual, SRS-NFR-03:start:end -->
