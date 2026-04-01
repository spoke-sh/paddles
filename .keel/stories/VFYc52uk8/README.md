---
# system-managed
id: VFYc52uk8
status: backlog
created_at: 2026-04-01T09:26:06
updated_at: 2026-04-01T09:28:09
# authored
title: Render TUI Transcript From The Canonical Conversation Plane
type: feat
operator-signal:
scope: VFYbtfpVG/VFYc27reW
index: 2
---

# Render TUI Transcript From The Canonical Conversation Plane

## Summary

Move the TUI transcript bootstrap and live update logic onto the canonical conversation transcript plane. The TUI should render the same shared conversation transcript as the web UI instead of relying on local-only append paths or global external trace scraping.

## Acceptance Criteria

- [ ] TUI transcript bootstrap uses canonical conversation replay [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Turns entered from the web UI appear in the TUI transcript for the same conversation without restart [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
- [ ] TUI transcript updates reconcile to the canonical conversation plane instead of relying on local-only transcript append state [SRS-05/AC-03] <!-- verify: review, SRS-05:start:end -->
