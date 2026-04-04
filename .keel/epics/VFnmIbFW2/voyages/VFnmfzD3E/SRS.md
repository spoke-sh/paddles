# Emit and Render Applied Edit Diffs - SRS

## Summary

Epic: VFnmIbFW2
Goal: Show applied workspace changes as first-class diff artifacts across the runtime, web UI, and TUI so workspace editor agency is obvious.

## Scope

### In Scope

- [SCOPE-01] Extend workspace editor action results so successful edits carry structured diff content and file metadata.
- [SCOPE-02] Emit a shared runtime applied-edit artifact for successful `apply_patch`, `replace_in_file`, and `write_file` actions.
- [SCOPE-03] Render applied-edit artifacts in the web runtime stream.
- [SCOPE-04] Render applied-edit artifacts in the TUI transcript stream.

### Out of Scope

- [SCOPE-05] Provider-specific edit rendering paths or model-specific diff UX.
- [SCOPE-06] New commit, review, or merge flows beyond showing the applied edit.
- [SCOPE-07] Rich multi-file browsing workflows outside the current interactive stream surfaces.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Workspace editor success paths for `apply_patch`, `replace_in_file`, and `write_file` SHALL return structured applied-edit data that includes affected file paths and diff content. | SCOPE-01 | FR-01 | test |
| SRS-02 | The runtime model SHALL emit a shared applied-edit artifact when the workspace editor reports a successful edit. | SCOPE-02 | FR-04 | test |
| SRS-03 | The web runtime stream SHALL render the applied-edit artifact with file identity and diff hunks instead of only a generic tool summary. | SCOPE-03 | FR-02 | test |
| SRS-04 | The TUI transcript stream SHALL render the applied-edit artifact with the same semantic content as the web stream. | SCOPE-04 | FR-03 | test |
| SRS-05 | Projection and contract tests SHALL cover the applied-edit artifact shape and the budget/runtime surfaces that reference it. | SCOPE-04 | FR-04 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Applied-edit visibility SHALL remain provider-agnostic by sourcing diff payloads from the workspace editor boundary and shared runtime events. | SCOPE-01, SCOPE-02 | NFR-02 | test |
| SRS-NFR-02 | Diff presentation SHALL remain compact enough for interactive runtime streams while preserving the changed lines needed for operator trust. | SCOPE-03, SCOPE-04 | NFR-03 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
