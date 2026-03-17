---
id: VE608nyGK
title: Migrate Registry to Sift
type: feat
status: done
created_at: 2026-03-16T21:45:15
started_at: 2026-03-16T21:50:00
updated_at: 2026-03-16T21:33:11
operator-signal: 
scope: VE5zxrA1w/VE604BPRi
index: 1
submitted_at: 2026-03-16T21:33:06
completed_at: 2026-03-16T21:33:11
---

# Migrate Registry to Sift

## Summary

Use `sift::internal` components to handle model asset acquisition.

## Acceptance Criteria

- [x] `SiftRegistryAdapter` replaces `HFHubAdapter`. [SRS-26/AC-01] <!-- verify: manual, SRS-26:start:end, proof: ac-1.log-->
- [x] No direct `hf-hub` imports in `paddles`. [SRS-NFR-11/AC-01] <!-- verify: manual, SRS-NFR-11:start:end, proof: ac-2.log-->
