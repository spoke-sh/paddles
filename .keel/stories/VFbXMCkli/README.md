---
# system-managed
id: VFbXMCkli
status: done
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T22:33:17
# authored
title: Render Context-Lineage-First Forensic Inspector In The Web UI
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 5
started_at: 2026-04-01T22:15:17
submitted_at: 2026-04-01T22:33:14
completed_at: 2026-04-01T22:33:17
---

# Render Context-Lineage-First Forensic Inspector In The Web UI

## Summary

Build the dense web forensic inspector around context lineage as the primary navigation model. The browser should let operators move across turn structure and artifact lineage in one unified surface and toggle between exact raw content and a format-friendly rendered view.

## Acceptance Criteria

- [x] The web UI presents unified context-lineage-first navigation across conversation, turn, model call, planner loop step, trace record, and artifact sequence [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] The selected artifact can toggle between exact raw content and a format-friendly rendered view [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end, proof: ac-2.log-->
- [x] Exact provider envelopes are inspectable in the UI with redacted sensitive fields and readable structure [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [x] The dense inspector remains usable for long conversations and large artifact payloads through local scrolling and focused panes rather than page-level scrolling [SRS-NFR-03/AC-04] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-4.log-->
