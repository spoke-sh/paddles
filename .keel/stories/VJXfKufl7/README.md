---
# system-managed
id: VJXfKufl7
status: backlog
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:29:36
# authored
title: Document Shared Planner Action Schema Contract
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4iAYl
index: 1
---

# Document Shared Planner Action Schema Contract

## Summary

Update foundational documentation so the shared planner action schema renderer,
turn-specific capability manifest, and provider adapter responsibilities match
the implemented runtime.

## Acceptance Criteria

- [ ] README describes the shared schema renderer and turn-specific capability manifest split. [SRS-01/AC-01] <!-- verify: review, SRS-01:start:end -->
- [ ] POLICY forbids adapter-local planner action schema lists. [SRS-02/AC-02] <!-- verify: review, SRS-02:start:end -->
- [ ] ARCHITECTURE maps the shared renderer boundary and provider adapter responsibilities. [SRS-03/AC-03] <!-- verify: review, SRS-03:start:end -->
- [ ] Docs mention semantic actions and `external_capability` as part of the canonical schema when capability-disclosed. [SRS-04/AC-04] <!-- verify: review, SRS-04:start:end -->
