---
id: VE608nyGK
title: Migrate Registry to Sift
type: feat
status: backlog
created_at: 2026-03-16T21:45:15
updated_at: 2026-03-16T21:31:17
operator-signal: 
scope: VE5zxrA1w/VE604BPRi
index: 1
---

# Migrate Registry to Sift

## Summary

Use `sift::internal` components to handle model asset acquisition.

## Acceptance Criteria

- [ ] `SiftRegistryAdapter` replaces `HFHubAdapter`. [SRS-26/AC-01] <!-- verify: manual, SRS-26:start:end -->
- [ ] No direct `hf-hub` imports in `paddles`. [SRS-NFR-11/AC-01] <!-- verify: manual, SRS-NFR-11:start:end -->
