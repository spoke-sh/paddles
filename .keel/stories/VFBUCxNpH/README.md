---
# system-managed
id: VFBUCxNpH
status: done
created_at: 2026-03-28T10:30:34
updated_at: 2026-03-28T13:20:29
# authored
title: Route Retrieval-Heavy Turns Through Context Gathering
type: feat
operator-signal:
scope: VFBTXlHli/VFBTYpPo6
index: 3
started_at: 2026-03-28T13:16:30
submitted_at: 2026-03-28T13:20:26
completed_at: 2026-03-28T13:20:29
---

# Route Retrieval-Heavy Turns Through Context Gathering

## Summary

Add controller routing that detects retrieval-heavy requests, invokes the
context-gathering lane, and feeds the resulting evidence bundle into final
answer synthesis.

## Acceptance Criteria

- [x] Retrieval-heavy requests are classified and routed through context gathering before final synthesis. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Non-retrieval requests, or retrieval-heavy requests whose gatherer lane is unavailable, preserve the existing default answer/tool path. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [x] Gatherer routing degrades safely and preserves deterministic workspace actions when the gatherer lane fails or is unsupported. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
