---
# system-managed
id: VHaVRxBd9
status: backlog
created_at: 2026-04-22T22:10:04
updated_at: 2026-04-22T22:12:25
# authored
title: Add Hosted Consumer Cursor Resume For Session Readers
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcrQZr
index: 1
---

# Add Hosted Consumer Cursor Resume For Session Readers

## Summary

Add hosted consumer cursor ownership for session and lifecycle readers so
service restart can resume from authoritative hosted positions instead of
replaying from zero by default.

## Acceptance Criteria

- [ ] Session and lifecycle consumers persist hosted cursor positions and resume from them on restart. [SRS-01/AC-01] <!-- verify: cargo test hosted_session_cursor_resume_ -- --nocapture, SRS-01:start:end -->
- [ ] Session readers expose resumed hosted cursor identity and position metadata for the running service instance. [SRS-01/AC-02] <!-- verify: cargo test hosted_session_cursor_resume_metadata_is_exposed -- --nocapture, SRS-01:start:end -->
