---
# system-managed
id: VFJ5wTEei
status: backlog
created_at: 2026-03-29T17:44:22
updated_at: 2026-03-29T17:47:05
# authored
title: Replace Lexical Interpretation Scoring With Model-Judged Guidance Selection
type: feat
operator-signal:
scope: VFJ5rdPZP/VFJ5t0Pbk
index: 2
---

# Replace Lexical Interpretation Scoring With Model-Judged Guidance Selection

## Summary

Replace lexical interpretation relevance scoring and ranked hint/procedure
selection with constrained model judgement so `AGENTS.md` roots and their
referenced guidance graph determine what memory matters for the current turn.

## Acceptance Criteria

- [ ] Interpretation-time guidance selection no longer depends on lexical term scoring for relevance on the primary path. [SRS-02/AC-01] <!-- verify: automated, SRS-02:start:end -->
- [ ] Tool hints and decision procedures are selected from the model-derived guidance graph rather than controller keyword ranking. [SRS-02/AC-02] <!-- verify: automated, SRS-02:start:end -->
- [ ] `AGENTS.md` remains the only hardcoded interpretation root. [SRS-NFR-02/AC-03] <!-- verify: automated, SRS-NFR-02:start:end -->
