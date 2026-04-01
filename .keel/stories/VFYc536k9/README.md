---
# system-managed
id: VFYc536k9
status: backlog
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T09:28:09
# authored
title: Render Web Transcript From The Canonical Conversation Plane
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 3
---

# Render Web Transcript From The Canonical Conversation Plane

## Summary

Move the browser transcript bootstrap and live update logic onto the canonical conversation transcript plane. The web UI should stop treating local POST responses, replay polling, and progress-event timing as transcript truth and instead reconcile to the application-owned conversation projection.

## Acceptance Criteria

- [ ] Web transcript bootstrap uses canonical conversation replay [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] Turns entered from TUI or CLI appear in the web transcript for the same conversation without page reload [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
- [ ] Web transcript updates reconcile to the canonical conversation plane instead of treating local POST/DOM state as transcript truth [SRS-04/AC-03] <!-- verify: review, SRS-04:start:end -->
