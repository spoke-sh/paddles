---
# system-managed
id: VGcZTGZgd
status: done
created_at: 2026-04-12T16:09:43
updated_at: 2026-04-12T16:47:19
# authored
title: Route External Capability Discovery Through The Recursive Harness
type: feat
operator-signal:
scope: VGb1c1XAL/VGcZRpCKi
index: 2
started_at: 2026-04-12T16:29:59
submitted_at: 2026-04-12T16:47:19
completed_at: 2026-04-12T16:47:19
---

# Route External Capability Discovery Through The Recursive Harness

## Summary

Teach the recursive planner and runtime loop to discover, select, and invoke
external capabilities through the same action flow used for local workspace and
shell work, including the first pass that normalizes external results into the
evidence-first runtime.

## Acceptance Criteria

- [x] The planner and runtime can discover external capabilities and invoke them through the same recursive action loop used for local work. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] External results normalize into evidence items, source records, and runtime artifacts with lineage, summaries, and availability state instead of remaining opaque tool output. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] External capability invocation composes with auth, approval, and sandbox governance instead of bypassing the local execution policy model. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] The recursive harness remains useful when all external capability fabrics are absent or disabled. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-4.log-->
