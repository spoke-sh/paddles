---
# system-managed
id: VHkhpORAN
status: icebox
created_at: 2026-04-24T16:01:58
updated_at: 2026-04-24T16:06:16
# authored
title: Add Safe Replacement And Edit Diagnostics
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgLtij9
index: 2
---

# Add Safe Replacement And Edit Diagnostics

## Summary

Add deterministic replacement fallbacks plus formatter and diagnostic evidence for workspace edits.

## Acceptance Criteria

- [ ] Ambiguous replacement attempts return candidate context instead of applying an unsafe edit. [SRS-02/AC-01] <!-- verify: cargo test workspace_replace_ambiguous -- --nocapture, SRS-02:start:end -->
- [ ] Formatter and diagnostic outcomes attach to edit evidence when configured and degrade gracefully when unavailable. [SRS-03/AC-01] <!-- verify: cargo test workspace_edit_diagnostics -- --nocapture, SRS-03:start:end -->
