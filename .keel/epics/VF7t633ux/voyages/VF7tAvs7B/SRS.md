# Sift-Native Runtime Cutover - SRS

## Summary

Epic: VF7t633ux
Goal: Replace legacy-engine-owned core orchestration with a Sift-backed controller that supports retained context and immediate local tool execution.

## Scope

### In Scope

- [SCOPE-01] Replace application/runtime ownership of prompt execution with a Sift-backed session controller.
- [SCOPE-02] Preserve prior turns, retained evidence, and tool outputs through Sift context abstractions.
- [SCOPE-03] Introduce immediate local tools for search, file listing/reading/editing, shell commands, and diffs.
- [SCOPE-04] Cut over the CLI and docs/dependencies to the new runtime boundary.

### Out of Scope

- [SCOPE-05] Remote tools or non-local execution services.
- [SCOPE-06] Fully autonomous planning or decomposition beyond the local tool loop.
- [SCOPE-07] Rich streaming or TUI work beyond preserving the existing terminal interaction pattern.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | `MechSuitService` must execute prompts through a Paddles-owned Sift session controller instead of `legacy_core::PromptLoop` and `Instance`. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The runtime must retain agent turns and bounded workspace evidence between prompts using Sift retained artifacts and local context sources. | SCOPE-02 | FR-01 | manual |
| SRS-03 | The runtime must expose immediate local tools for search, file operations, shell commands, and edit/diff operations. | SCOPE-03 | FR-02 | manual |
| SRS-04 | Each executed tool result must be recorded as searchable local context for later turns. | SCOPE-02, SCOPE-03 | FR-03 | manual |
| SRS-05 | Single-prompt and interactive CLI flows must remain operational after the runtime cutover. | SCOPE-04 | FR-04 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | legacy-core/provider/tools must be removed from core runtime modules and Cargo runtime dependencies. | SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | Verbose execution must report context assembly and tool activity for debugging. | SCOPE-01, SCOPE-03 | NFR-03 | manual |
| SRS-NFR-03 | The controller and tool path must remain local-first with no new network dependency on prompt execution. | SCOPE-01, SCOPE-03 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
