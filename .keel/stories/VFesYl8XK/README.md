---
# system-managed
id: VFesYl8XK
status: backlog
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:10:13
# authored
title: Add A Dedicated Steering Signal Manifold Route To The Web UI
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 2
---

# Add A Dedicated Steering Signal Manifold Route To The Web UI

## Summary

Add the dedicated web route and shell for the steering signal manifold. This slice establishes the separate surface, preserves the existing precise forensic inspector, and gives the manifold route room for its own canvas, timeline, and detail panes.

## Acceptance Criteria

- [ ] The web UI exposes a dedicated steering signal manifold route distinct from the precise forensic inspector [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] The route layout makes the manifold visualization primary on that route while preserving room for detail and source panes [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] The route remains usable for long conversations through bounded local scrolling and without page-level overflow churn [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
