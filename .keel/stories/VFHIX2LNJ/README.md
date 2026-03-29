---
# system-managed
id: VFHIX2LNJ
status: backlog
created_at: 2026-03-29T10:21:50
updated_at: 2026-03-29T10:25:12
# authored
title: Render Threaded Transcript And Merge-Back UX
type: feat
operator-signal:
scope: VFHIUOcFc/VFHIV59Hn
index: 5
---

# Render Threaded Transcript And Merge-Back UX

## Summary

Extend the default transcript so operators can see when a steering prompt stays
on the mainline, opens a child thread, or merges back, without turning the TUI
into a raw recorder dump.

## Acceptance Criteria

- [ ] The default transcript surfaces thread split, active-thread state, and merge-back outcomes clearly enough to follow live. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Thread-local and mainline context remain visually distinguishable without overwhelming the transcript. [SRS-NFR-03/AC-02] <!-- verify: manual, SRS-NFR-03:start:end -->
- [ ] Merge-back rendering uses explicit recorded outcomes instead of implying hidden history rewrites. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end -->
