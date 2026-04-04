# Workspace Editor Diff Visibility - Product Requirements

## Problem Statement

Operators cannot see when Paddles actually edited the workspace because workspace editor actions collapse into generic tool summaries instead of first-class diffs.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make workspace editor actions immediately legible to operators as applied diffs instead of generic tool chatter. | On an edit turn, the runtime stream shows a first-class diff artifact with file identity and hunk content instead of only a tool summary. | Achieve on both web and TUI surfaces for applied-edit turns in this voyage. |
| GOAL-02 | Preserve the architectural boundary between edit execution and provider behavior while adding the new visibility surface. | The emitted diff artifact originates from workspace editor results and shared runtime events, with no provider-specific edit rendering branch. | All applied-edit rendering paths consume the same shared artifact contract. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | A developer using Paddles interactively through the TUI or web runtime surfaces. | Immediate proof that Paddles actually changed the workspace, with enough diff detail to trust the action. |
| Reviewer | A person validating mission completion or turn behavior after the fact. | A stable applied-edit artifact that can be inspected without reconstructing tool logs by hand. |

## Scope

### In Scope

- [SCOPE-01] Extend workspace editor results and runtime events so applied edits carry structured diff data.
- [SCOPE-02] Render applied-edit diff artifacts in the web runtime stream.
- [SCOPE-03] Render the same applied-edit diff artifacts in the TUI transcript stream.
- [SCOPE-04] Lock the behavior with projection, contract, and UI tests.

### Out of Scope

- [SCOPE-05] New edit-authoring models or provider-specific edit UX.
- [SCOPE-06] Rich code review workflows beyond showing the applied diff itself.
- [SCOPE-07] Multi-file staging, commit review, or merge tooling outside the existing workspace editor action set.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Emit a structured applied-edit artifact whenever the workspace editor successfully completes `apply_patch`, `replace_in_file`, or `write_file`. | GOAL-01, GOAL-02 | must | Makes edit execution visible through a shared runtime contract instead of prose summaries. |
| FR-02 | Render applied-edit artifacts in the web runtime stream with file identity and diff hunks. | GOAL-01 | must | Gives operators immediate visual proof of the edit in the web UI. |
| FR-03 | Render applied-edit artifacts in the TUI transcript stream with the same semantic content as the web UI. | GOAL-01 | must | Keeps interactive operator trust consistent across surfaces. |
| FR-04 | Surface enough metadata on each applied-edit artifact to support projection and verification tests. | GOAL-01, GOAL-02 | should | Prevents the diff surface from becoming a UI-only side channel. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Maintain reliability and observability for all new workflow paths introduced by this epic. | GOAL-01, GOAL-02 | must | Keeps operations stable and makes regressions detectable during rollout. |
| NFR-02 | Keep edit visibility provider-agnostic by sourcing diff output from the workspace editor boundary rather than model-specific adapters. | GOAL-02 | must | Protects the architecture from regressing into provider-specific fast paths. |
| NFR-03 | Keep diff rendering compact enough for interactive streams while preserving the changed lines needed for operator trust. | GOAL-01 | should | The artifact needs to be readable in-line, not just technically present. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Workspace editor artifact | Rust unit and integration tests on workspace editor results and runtime event emission | Story-level test evidence showing structured diff payloads for patch, replace, and write actions |
| Web rendering | Frontend tests plus captured runtime projection output | Story-level evidence showing diff cards or rows in the web stream |
| TUI rendering | Transcript or projection contract tests plus manual proof if needed | Story-level evidence showing diff presentation in the TUI stream |
| Problem outcome | End-to-end turn proof on an edit request | Mission evidence showing the operator can see the applied diff without reconstructing tool chatter |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| A unified diff-like artifact is the right minimal proof format for operator trust. | The UI may still feel opaque even if an edit artifact exists. | Validate against the requested Codex/Claude/Gemini-style expectation during story acceptance. |
| Existing workspace editor actions can provide enough before/after context to synthesize stable diffs. | Additional edit-specific storage or capture may be required. | Prove against `apply_patch`, `replace_in_file`, and `write_file` in the first story slice. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| How much hunk detail should be rendered inline before collapsing or truncating? | Epic owner | Open |
| `write_file` may need synthetic before/after diff generation that differs from `apply_patch` passthrough semantics. | Implementer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] An edit turn that uses the workspace editor emits a structured applied-edit artifact instead of only a generic tool summary.
- [ ] The web runtime stream shows the applied diff with file identity and changed lines.
- [ ] The TUI transcript stream shows the same applied diff semantics.
- [ ] The applied-edit artifact is covered by automated tests and can be used as mission completion evidence.
<!-- END SUCCESS_CRITERIA -->
