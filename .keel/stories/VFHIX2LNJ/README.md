---
# system-managed
id: VFHIX2LNJ
status: done
created_at: 2026-03-29T10:21:50
updated_at: 2026-03-29T11:58:34
# authored
title: Render Threaded Transcript And Merge-Back UX
type: feat
operator-signal:
scope: VFHIUOcFc/VFHIV59Hn
index: 5
started_at: 2026-03-29T11:57:03
submitted_at: 2026-03-29T11:58:33
completed_at: 2026-03-29T11:58:34
---

# Render Threaded Transcript And Merge-Back UX

## Summary

Extend the default transcript so operators can see when a steering prompt stays
on the mainline, opens a child thread, or merges back, without turning the TUI
into a raw recorder dump.

## Acceptance Criteria

- [x] The default transcript surfaces thread split, active-thread state, and merge-back outcomes clearly enough to follow live. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] Thread-local and mainline context remain visually distinguishable without overwhelming the transcript. [SRS-NFR-03/AC-02] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-2.log-->
- [x] Merge-back rendering uses explicit recorded outcomes instead of implying hidden history rewrites. [SRS-06/AC-03] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
