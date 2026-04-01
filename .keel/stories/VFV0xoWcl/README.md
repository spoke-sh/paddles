---
# system-managed
id: VFV0xoWcl
status: backlog
created_at: 2026-03-31T18:39:51
updated_at: 2026-03-31T18:41:56
# authored
title: Rename And Rewire Gatherer Configuration Away From Autonomous Planning
type: feat
operator-signal:
scope: VFV0VmEj0/VFV0uvpPX
index: 3
---

# Rename And Rewire Gatherer Configuration Away From Autonomous Planning

## Summary

Align configuration, provider naming, and runtime wiring with the new architecture so paddles operators see sift as a retrieval backend rather than a second planner.

## Acceptance Criteria

- [ ] Gatherer configuration and provider selection no longer imply that paddles delegates recursive planning to `sift-autonomous`. [SRS-05/AC-01] <!-- verify: test, SRS-05:start:end, proof: ac-1.log-->
- [ ] Runtime labels and summaries describe sift as a retrieval backend in logs, traces, or UI copy where applicable. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [ ] Any required compatibility aliasing or migration behavior is explicit rather than silently preserving misleading autonomous terminology. [SRS-05/AC-03] <!-- verify: test, SRS-05:start:end, proof: ac-3.log-->
