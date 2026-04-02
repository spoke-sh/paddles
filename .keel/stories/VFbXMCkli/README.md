---
# system-managed
id: VFbXMCkli
status: backlog
created_at: 2026-04-01T21:26:10
updated_at: 2026-04-01T21:29:42
# authored
title: Render Context-Lineage-First Forensic Inspector In The Web UI
type: feat
operator-signal:
scope: VFbXKEdWb/VFbXKFBWT
index: 5
---

# Render Context-Lineage-First Forensic Inspector In The Web UI

## Summary

Build the dense web forensic inspector around context lineage as the primary navigation model. The browser should let operators move across turn structure and artifact lineage in one unified surface and toggle between exact raw content and a format-friendly rendered view.

## Acceptance Criteria

- [ ] The web UI presents unified context-lineage-first navigation across conversation, turn, model call, planner loop step, trace record, and artifact sequence [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] The selected artifact can toggle between exact raw content and a format-friendly rendered view [SRS-06/AC-02] <!-- verify: manual, SRS-06:start:end -->
- [ ] Exact provider envelopes are inspectable in the UI with redacted sensitive fields and readable structure [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
- [ ] The dense inspector remains usable for long conversations and large artifact payloads through local scrolling and focused panes rather than page-level scrolling [SRS-NFR-03/AC-04] <!-- verify: manual, SRS-NFR-03:start:end -->
