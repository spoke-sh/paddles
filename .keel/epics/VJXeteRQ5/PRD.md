# Shared Planner Action Schema Contract - Product Requirements

## Problem Statement

Planner action JSON schemas are duplicated inside individual planner adapters even though Rust action enums and runtime capability contracts are shared. This allows Sift, HTTP, retry, redecision, semantic actions, and external capability prompts to drift from the canonical bounded action contract.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make the planner action schema a single authored contract rendered by every planner lane. | No adapter owns a separate planner action JSON example list or field requirement list. | All planner prompt paths call the shared renderer. |
| GOAL-02 | Prove the authored schema matches the Rust action surface. | Tests fail when `InitialAction`, `PlannerAction`, or `WorkspaceAction` gains or loses a variant without a schema update. | Enum/schema parity tests cover common, semantic, and external actions. |
| GOAL-03 | Prove Sift/local and HTTP/remote planners receive the same canonical action schema. | Mocked planner-turn tests extract and compare the canonical schema block from both lanes. | Initial and recursive prompts match across Sift and HTTP. |
| GOAL-04 | Keep foundational docs aligned with the runtime contract. | README, POLICY, ARCHITECTURE, and owning docs describe the shared schema renderer and capability manifest split. | Docs reviewed in the same delivery slice. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Paddles operator | Runs Paddles as a coding agent and reads traces/prompts to understand planner behavior. | The planner action surface is clear, stable, and consistent across model lanes. |
| Paddles maintainer | Adds or changes planner actions, providers, and tests. | One place to update the prompt-facing action schema, with tests catching drift. |
| Library integrator | Embeds Paddles as a Rust library or configures alternate planner lanes. | The same bounded action contract applies regardless of provider adapter. |

## Scope

### In Scope

- [SCOPE-01] A shared authored planner action schema contract and renderer.
- [SCOPE-02] Enum/schema parity tests for initial, recursive, workspace,
  semantic, and external actions.
- [SCOPE-03] Sift and HTTP prompt paths migrated to the shared renderer.
- [SCOPE-04] End-to-end mocked turns proving both Sift and HTTP receive the
  same canonical schema for initial and recursive planner prompts.
- [SCOPE-05] Foundational documentation updates for the shared schema contract.

### Out of Scope

- [SCOPE-06] Adding new planner actions beyond action variants already present
  in the Rust action surface.
- [SCOPE-07] Changing local-first execution governance, sandbox behavior, or
  provider authentication.
- [SCOPE-08] Replacing the planner / response-lane architectural split.
- [SCOPE-09] Rewriting prompt transports beyond delegating action-schema text to
  the shared renderer.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Provide one shared renderer for planner action schema text, including JSON examples, required fields, and common action-selection rules. | GOAL-01 | must | Removes adapter-local schema ownership. |
| FR-02 | Cover `answer`, `search`, `list_files`, `read`, `inspect`, `shell`, `diff`, `write_file`, `replace_in_file`, `apply_patch`, semantic workspace actions, `external_capability`, `refine`, `branch`, and `stop`. | GOAL-01, GOAL-02 | must | The schema must expose the full current bounded action surface. |
| FR-03 | Keep runtime capability availability separate from schema definition through the existing planner execution contract / capability manifest. | GOAL-01 | must | The model sees both the stable action vocabulary and turn-specific availability. |
| FR-04 | Replace Sift and HTTP hardcoded action-schema blocks with shared-renderer output while preserving provider-specific transport instructions. | GOAL-01, GOAL-03 | must | Avoids drift without flattening real transport differences. |
| FR-05 | Add mocked end-to-end turn tests that prove Sift/local and HTTP/remote planner prompts include the same canonical schema. | GOAL-03 | must | The acceptance bar is behavior-level parity, not just helper unit tests. |
| FR-06 | Update foundational docs that own planner action contracts and turn-loop behavior. | GOAL-04 | must | Behavior and operator-facing explanation must stay synchronized. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve local-first execution and fail-closed governance semantics. | GOAL-01 | must | Schema unification must not widen authority. |
| NFR-02 | Keep provider-specific transport prompt text isolated from action-schema text. | GOAL-01, GOAL-03 | should | Providers can differ in JSON transport while sharing action vocabulary. |
| NFR-03 | Make schema drift failures easy to diagnose in test output. | GOAL-02, GOAL-03 | should | Maintainers need clear failures when an action is added or omitted. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Schema contract | Unit tests over the authored contract and renderer | Test output showing all expected action names and field requirements |
| Enum parity | Tests comparing contract entries to Rust enum labels / supported variants | Failing test coverage for missing or extra variants |
| Lane parity | Mocked Sift and HTTP turns that extract the canonical schema block | Test output proving both lanes receive the same schema for initial and recursive prompts |
| Documentation | Diff review plus doc checks | README/POLICY/ARCHITECTURE references updated with shared schema contract |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Adapter-local prompt text is the primary source of schema drift. | Work may miss another schema duplication path. | `rg` for action JSON examples and prompt schema blocks before implementation closes. |
| An authored contract is preferable to deriving prompt text directly from enums. | More maintenance is required when action variants change. | Enum/schema parity tests make updates explicit and reviewable. |
| Sift and HTTP mocked turns can expose comparable prompt schema blocks. | Acceptance may need lower-level prompt construction tests. | Start with mocked-turn tests and fall back only if the transport boundary makes prompt extraction impossible. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Where should the shared renderer live so domain types stay clean while infrastructure adapters can reuse it? | Implementer | Open |
| Should semantic actions be always listed in the schema but marked capability-dependent through the manifest? | Implementer | Open |
| How should HTTP native tool schemas and prompt-envelope examples share the same contract without losing transport-specific validity? | Implementer | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] One shared planner action schema renderer is used by all planner prompt paths.
- [ ] Tests prove the authored schema matches `InitialAction`, `PlannerAction`, and `WorkspaceAction`.
- [ ] Mocked Sift and HTTP turns receive the same canonical schema for initial and recursive planner prompts.
- [ ] Adapter-local hardcoded action-schema lists are removed or reduced to calls into the shared renderer.
- [ ] Foundational documentation describes the shared schema contract and the separate turn-specific capability manifest.
<!-- END SUCCESS_CRITERIA -->
