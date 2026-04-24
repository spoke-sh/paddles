# Harden Workspace Editing And LSP Hands - SRS

## Summary

Epic: VHkfpJJc4
Goal: Make workspace edits and semantic code intelligence safe, diagnosable, and planner-accessible through production-grade edit behavior and LSP-backed workspace actions.

## Scope

### In Scope

- [SCOPE-03] Upgrade workspace edit hands with safer replacements, file locks, line-ending/BOM preservation, diff evidence, formatter hooks, and diagnostics.
- [SCOPE-04] Add semantic workspace intelligence through LSP-backed navigation and diagnostics as typed planner-accessible actions.

### Out of Scope

- [SCOPE-10] Building a full IDE.
- [SCOPE-10] Replacing existing apply-patch support.
- [SCOPE-11] Requiring an LSP server for basic file edits.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Preserve file encoding markers and line endings across write, replace, and patch operations. | SCOPE-03 | FR-03 | test: file preservation fixtures |
| SRS-02 | Add deterministic replacement fallbacks and per-file edit locking. | SCOPE-03 | FR-03 | test: replacement and concurrency tests |
| SRS-03 | Attach formatter and diagnostic results to edit evidence when configured. | SCOPE-03 | FR-03 | test: edit evidence tests |
| SRS-04 | Expose LSP-backed semantic actions through typed workspace capabilities. | SCOPE-04 | FR-04 | test: LSP adapter contract tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Edit behavior remains deterministic and auditable. | SCOPE-03 | NFR-04 | test: diff and evidence snapshots |
| SRS-NFR-02 | Missing formatter or LSP support degrades gracefully. | SCOPE-03, SCOPE-04 | NFR-01 | test: unavailable diagnostic posture |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
