---
# system-managed
id: VJXfKuNks
status: backlog
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:29:36
# authored
title: Prove Planner Lane Schema Parity
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hlYW
index: 2
---

# Prove Planner Lane Schema Parity

## Summary

Add mocked-turn tests that extract the canonical schema block from Sift and
HTTP planner prompts and compare the blocks exactly.

## Acceptance Criteria

- [ ] Mocked Sift and HTTP initial planner turns receive the same canonical schema block. [SRS-04/AC-01] <!-- verify: test, SRS-04:start:end -->
- [ ] Mocked Sift and HTTP recursive planner turns receive the same canonical schema block. [SRS-05/AC-02] <!-- verify: test, SRS-05:start:end -->
- [ ] Test failures identify the drifting lane and prompt variant. [SRS-NFR-02/AC-03] <!-- verify: test, SRS-NFR-02:start:end -->
