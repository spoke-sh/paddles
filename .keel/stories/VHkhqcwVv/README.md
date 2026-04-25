---
# system-managed
id: VHkhqcwVv
status: done
created_at: 2026-04-24T16:02:03
updated_at: 2026-04-24T18:28:36
# authored
title: Add LSP Semantic Workspace Actions
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgLtij9
index: 3
started_at: 2026-04-24T18:24:48
completed_at: 2026-04-24T18:28:36
---

# Add LSP Semantic Workspace Actions

## Summary

Expose LSP-backed semantic workspace actions for code navigation and diagnostics without requiring LSP for basic file operations.

## Acceptance Criteria

- [x] Workspace capabilities include typed semantic actions for definitions, references, symbols, hover, and diagnostics when LSP is available. [SRS-04/AC-01] <!-- verify: cargo test semantic_workspace_actions -- --nocapture, SRS-04:start:end, proof: ac-1.log-->
- [x] Missing LSP support returns unavailable semantic results while preserving search/read fallback paths. [SRS-NFR-02/AC-01] <!-- verify: cargo test semantic_workspace_unavailable -- --nocapture, SRS-NFR-02:start:end, proof: ac-2.log-->
