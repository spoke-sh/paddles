# Sift Controller Migration - Product Requirements

## Problem Statement

Paddles core execution is still owned by wonopcode rather than a Sift-backed local controller, which prevents local tool calling and searchable retained context from being first-class runtime concepts.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make Paddles execute coding prompts through a Sift-owned runtime instead of wonopcode-owned core orchestration. | `paddles --prompt` and interactive mode complete through the Sift controller with retained context | Verified CLI proof |
| GOAL-02 | Expose a practical local tool surface for coding work from the first Sift-native cut. | Search, file, shell, and edit/diff tools are callable and their outputs influence later turns | 5 tool families verified |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer or agent running `paddles` inside a workspace. | A local-first coding assistant that can gather context and act on files without wonopcode runtime ownership. |

## Scope

### In Scope

- [SCOPE-01] Replace `PromptLoop`/`Instance`-owned core execution with a Paddles-managed Sift session controller.
- [SCOPE-02] Preserve bounded retained evidence and prior turn/tool outputs through Sift context abstractions.
- [SCOPE-03] Ship immediate local tools for search, file listing/reading/editing, shell commands, and diff visibility.
- [SCOPE-04] Remove wonopcode-core/provider/tools from core runtime modules and update docs/dependency manifests to match the new boundary.

### Out of Scope

- [SCOPE-05] Remote tools, cloud execution, or any non-local execution substrate.
- [SCOPE-06] Autonomous task planning beyond the local tool loop required for prompt execution.
- [SCOPE-07] Rich streaming/TUI presentation work beyond keeping the existing CLI flows operational.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Paddles must execute prompt turns through a Sift-backed runtime session that owns turn history, retained artifacts, and tool output context. | GOAL-01 | must | This is the core cutover that removes wonopcode ownership from execution. |
| FR-02 | Paddles must expose immediate local tools for search, file inspection/editing, shell commands, and diffs. | GOAL-02 | must | Coding tasks require a broader local tool surface than retrieval alone. |
| FR-03 | Tool results must be recorded as searchable local context so later turns can reuse them without reconstructing state. | GOAL-01, GOAL-02 | must | This makes Sift's context abstraction materially useful in the runtime. |
| FR-04 | Existing single-prompt and interactive CLI workflows must continue to operate after the cutover. | GOAL-01 | must | The refactor cannot regress the primary user entrypoints. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | wonopcode-core/provider/tools must not remain in core runtime modules or the Cargo runtime dependency graph after the cutover. | GOAL-01 | must | Enforces the architectural boundary instead of leaving it aspirational. |
| NFR-02 | The new controller must stay local-first and add no new network dependency to prompt execution or tool execution. | GOAL-01, GOAL-02 | must | Preserves the project’s execution model and operational policy. |
| NFR-03 | Verbose execution must expose context assembly and tool activity clearly enough to debug controller behavior. | GOAL-01, GOAL-02 | should | The new controller needs inspectable behavior during migration. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Runtime cutover | `cargo check`, interactive/manual CLI proofs, and story-level evidence logs | Story verification artifacts and final CLI transcripts |
| Tool surface | Manual proofs for search, file, shell, and edit/diff tool paths | Story evidence logs with representative tool outputs |
| Dependency boundary | `cargo tree` and code review of runtime modules | Story evidence showing wonopcode removed from core runtime path |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Sift’s current embedding surface is sufficient for retained-context search and local generative sessions. | The runtime cutover may require upstream Sift changes before landing. | Validate during story `VF7tCKEgw`. |
| Local shell and file tooling is an acceptable first-class capability for Paddles. | The tool surface may need tighter constraints or a different safety model. | Validate during story `VF7tCKUgx`. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| The local tool protocol may need prompt tuning before the model reliably requests tools. | Operator | Open |
| A hard wonopcode cutover may require coordinated documentation and build manifest cleanup. | Manager | Planned |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] `paddles --prompt` and interactive mode run through a Sift-native controller with retained context.
- [ ] Paddles can use local search, file, shell, and edit/diff tools in the same runtime session.
- [ ] wonopcode-core/provider/tools are removed from core runtime code and Cargo runtime dependencies.
<!-- END SUCCESS_CRITERIA -->
