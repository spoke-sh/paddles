---
# system-managed
id: VGLDQ8NnO
status: backlog
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:15
# authored
title: Promote Embedded Transit Recorder To Default Session Spine
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuE5W
index: 2
---

# Promote Embedded Transit Recorder To Default Session Spine

## Summary

Promote the embedded transit-backed recorder path into the default session spine for the runtime. This story should bound how persistent session recording becomes the normal path without breaking local-first failure behavior.

## Acceptance Criteria

- [ ] The runtime defines a default recorder posture that uses the embedded transit-backed session spine instead of treating recording as optional metadata [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Fallback and failure behavior remains bounded and operator-visible when the persistent session spine cannot be used [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
