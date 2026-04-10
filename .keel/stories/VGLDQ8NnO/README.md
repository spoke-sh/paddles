---
# system-managed
id: VGLDQ8NnO
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T17:13:47
# authored
title: Promote Embedded Transit Recorder To Default Session Spine
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuE5W
index: 2
started_at: 2026-04-09T17:13:20
completed_at: 2026-04-09T17:13:47
---

# Promote Embedded Transit Recorder To Default Session Spine

## Summary

Promote the embedded transit-backed recorder path into the default session spine for the runtime. This story should bound how persistent session recording becomes the normal path without breaking local-first failure behavior.

## Acceptance Criteria

- [x] The runtime defines a default recorder posture that uses the embedded transit-backed session spine instead of treating recording as optional metadata [SRS-02/AC-01] <!-- verify: cargo test trace_recorders -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] Fallback and failure behavior remains bounded and operator-visible when the persistent session spine cannot be used [SRS-02/AC-02] <!-- verify: cargo test service_new_uses_persistent_trace_recorder_posture_by_default -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
