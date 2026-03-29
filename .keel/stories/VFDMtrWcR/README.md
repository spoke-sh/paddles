---
# system-managed
id: VFDMtrWcR
status: done
created_at: 2026-03-28T18:14:05
updated_at: 2026-03-28T18:48:16
# authored
title: Document And Prove Evidence-First Turn Behavior
type: feat
operator-signal:
scope: VFDMnu8k9/VFDMp3Zn3
index: 5
started_at: 2026-03-28T18:46:56
submitted_at: 2026-03-28T18:48:13
completed_at: 2026-03-28T18:48:16
---

# Document And Prove Evidence-First Turn Behavior

## Summary

Update the foundational docs and proof artifacts so the new evidence-first turn
model, default file citations, and default action stream are documented and
demonstrated end-to-end.

## Acceptance Criteria

- [x] Foundational docs explain that repository questions use an explicit gatherer-first path, default cited synthesis, and the default action stream. [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end, proof: ac-1.log-->
- [x] Proof artifacts compare the old weak hidden-retrieval behavior against the new evidence-first behavior on representative prompts. [SRS-07/AC-02] <!-- verify: manual, SRS-07:start:end, proof: ac-2.log-->
- [x] Operator-facing examples show the expected Codex-style transcript shape so future regressions are easy to spot. [SRS-07/AC-03] <!-- verify: manual, SRS-07:start:end, proof: ac-3.log-->
