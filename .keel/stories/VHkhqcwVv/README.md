---
# system-managed
id: VHkhqcwVv
status: icebox
created_at: 2026-04-24T16:02:03
updated_at: 2026-04-24T16:06:21
# authored
title: Add LSP Semantic Workspace Actions
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgLtij9
index: 3
---

# Add LSP Semantic Workspace Actions

## Summary

Expose LSP-backed semantic workspace actions for code navigation and diagnostics without requiring LSP for basic file operations.

## Acceptance Criteria

- [ ] Workspace capabilities include typed semantic actions for definitions, references, symbols, hover, and diagnostics when LSP is available. [SRS-04/AC-01] <!-- verify: cargo test semantic_workspace_actions -- --nocapture, SRS-04:start:end -->
- [ ] Missing LSP support returns unavailable semantic results while preserving search/read fallback paths. [SRS-NFR-02/AC-01] <!-- verify: cargo test semantic_workspace_unavailable -- --nocapture, SRS-NFR-02:start:end -->
