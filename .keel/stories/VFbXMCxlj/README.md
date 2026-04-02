---
# system-managed
id: VFbXMCxlj
status: done
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T22:44:16
# authored
title: Stream Provisional Active-Turn Inspector Artifacts
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 6
started_at: 2026-04-01T22:41:21
submitted_at: 2026-04-01T22:44:14
completed_at: 2026-04-01T22:44:16
---

# Stream Provisional Active-Turn Inspector Artifacts

## Summary

Make the inspector useful during active turns by streaming provisional forensic artifacts into the browser as they form. The UI should show them in coherent sequence, mark them as provisional, and reconcile them in place when superseded or finalized.

## Acceptance Criteria

- [x] Active turns show provisional forensic artifacts in coherent sequence as context is assembled and model responses arrive [SRS-09/AC-01] <!-- verify: manual, SRS-09:start:end, proof: ac-1.log-->
- [x] Provisional artifacts are clearly marked and reconcile in place when superseded or finalized [SRS-09/AC-02] <!-- verify: manual, SRS-09:start:end, proof: ac-2.log-->
- [x] Live provisional updates preserve replay coherence for lineage and force views instead of forking browser-only state [SRS-09/AC-03] <!-- verify: cargo test -q forensic_inspector_html_subscribes_to_replay_backed_live_updates, SRS-09:start:end, proof: ac-3.log-->
- [x] The rollout remains web-only and does not require matching TUI inspector changes [SRS-NFR-05/AC-04] <!-- verify: manual, SRS-NFR-05:start:end, proof: ac-4.log-->
