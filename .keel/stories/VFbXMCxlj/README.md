---
# system-managed
id: VFbXMCxlj
status: backlog
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T21:29:42
# authored
title: Stream Provisional Active-Turn Inspector Artifacts
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 6
---

# Stream Provisional Active-Turn Inspector Artifacts

## Summary

Make the inspector useful during active turns by streaming provisional forensic artifacts into the browser as they form. The UI should show them in coherent sequence, mark them as provisional, and reconcile them in place when superseded or finalized.

## Acceptance Criteria

- [ ] Active turns show provisional forensic artifacts in coherent sequence as context is assembled and model responses arrive [SRS-09/AC-01] <!-- verify: manual, SRS-09:start:end -->
- [ ] Provisional artifacts are clearly marked and reconcile in place when superseded or finalized [SRS-09/AC-02] <!-- verify: manual, SRS-09:start:end -->
- [ ] Live provisional updates preserve replay coherence for lineage and force views instead of forking browser-only state [SRS-09/AC-03] <!-- verify: test, SRS-09:start:end -->
- [ ] The rollout remains web-only and does not require matching TUI inspector changes [SRS-NFR-05/AC-04] <!-- verify: review, SRS-NFR-05:start:end -->
