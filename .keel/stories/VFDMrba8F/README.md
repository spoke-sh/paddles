---
# system-managed
id: VFDMrba8F
status: done
created_at: 2026-03-28T18:13:57
updated_at: 2026-03-28T18:48:16
# authored
title: Tighten Turn Classification For Retrieval And Action Intents
type: feat
operator-signal:
scope: VFDMnu8k9/VFDMp3Zn3
index: 2
started_at: 2026-03-28T18:46:56
submitted_at: 2026-03-28T18:48:13
completed_at: 2026-03-28T18:48:16
---

# Tighten Turn Classification For Retrieval And Action Intents

## Summary

Strengthen turn classification so `paddles` can reliably distinguish casual
chat, deterministic actions, repository questions, and deeper
decomposition/research turns before choosing lanes or tools.

## Acceptance Criteria

- [x] The controller can distinguish at least casual, deterministic action, repository question, and decomposition/research intents using stable runtime logic. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] The classification decision is exposed as a typed turn event or equivalent runtime signal before gatherer, tool, or synthesis work begins. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-2.log-->
- [x] Regression coverage includes natural repository questions such as `How does memory work in paddles?` so they do not fall back to weak generic chat handling. [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end, proof: ac-3.log-->
