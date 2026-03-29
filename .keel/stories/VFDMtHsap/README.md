---
# system-managed
id: VFDMtHsap
status: backlog
created_at: 2026-03-28T18:14:03
updated_at: 2026-03-28T18:17:19
# authored
title: Add Default Codex-Style Turn Event Stream
type: feat
operator-signal:
scope: VFDMnu8k9/VFDMp3Zn3
index: 4
---

# Add Default Codex-Style Turn Event Stream

## Summary

Add a typed, default-on REPL event stream that renders Codex-style action lines
for classification, retrieval, planner work, tool calls, fallbacks, and final
synthesis.

## Acceptance Criteria

- [ ] The default REPL output renders a Codex-style turn stream covering the major execution steps for each turn, not just debug-only backend logs. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] The event stream can represent gatherer, planner, tool, fallback, synthesis, and any remaining synthesizer-side retrieval events with concise summaries and bounded detail so visible execution matches runtime behavior. [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] The stream remains the default interactive UX with no quiet flag introduced as part of this change. [SRS-05/AC-03] <!-- verify: manual, SRS-05:start:end -->
