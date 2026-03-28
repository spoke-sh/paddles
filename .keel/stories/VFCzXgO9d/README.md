---
# system-managed
id: VFCzXgO9d
status: backlog
created_at: 2026-03-28T16:41:19
updated_at: 2026-03-28T16:43:21
# authored
title: Route Multi-Hop Retrieval Through Autonomous Planning
type: feat
operator-signal:
scope: VFCzL9KKd/VFCzWHL1Y
index: 3
---

# Route Multi-Hop Retrieval Through Autonomous Planning

## Summary

Teach the controller to route decomposition-worthy repository-investigation
prompts through the autonomous gatherer lane while preserving the current
synthesizer-first path for ordinary chat, coding, and deterministic tool turns.

## Acceptance Criteria

- [ ] Controller routing distinguishes decomposition-worthy prompts from ordinary chat/tool turns and selects the autonomous gatherer only when appropriate. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [ ] Prompts that do not need autonomous planning, or turns where the autonomous gatherer is unavailable, remain on the current synthesizer-first path with clear fallback behavior. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [ ] Integration tests or CLI proofs demonstrate a multi-hop investigation prompt using autonomous retrieval planning before final synthesis. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
