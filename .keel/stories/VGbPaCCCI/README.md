---
# system-managed
id: VGbPaCCCI
status: done
created_at: 2026-04-12T11:24:10
updated_at: 2026-04-12T12:19:36
# authored
title: Project Live Control State Across Operator Surfaces
type: feat
operator-signal:
scope: VGb1c1AAK/VGbPWnUh2
index: 3
started_at: 2026-04-12T12:08:32
submitted_at: 2026-04-12T12:19:31
completed_at: 2026-04-12T12:19:36
---

# Project Live Control State Across Operator Surfaces

## Summary

Project live control state, plan updates, diff state, command summaries, and
file-change artifacts across transcript, TUI, web, and API surfaces so a
running turn becomes legible without reading raw trace internals.

## Acceptance Criteria

- [x] Active turns emit typed runtime items for plan updates, diff updates, command summaries, file changes, and control-state transitions. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] TUI, web, and API projections render one shared control and runtime-item vocabulary without divergent semantics. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] Invalid or stale control requests surface explicit rejected, stale, or unavailable status instead of mutating hidden thread state. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [x] Transcript and UI surfaces keep control transitions readable and surface degraded or unsupported states honestly. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-4.log-->
- [x] Shared control-state projections remain deterministic enough for focused replay and projection proofs. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-5.log-->
- [x] The resulting control plane preserves the local-first recursive execution model while exposing live operator state. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end, proof: ac-6.log-->
