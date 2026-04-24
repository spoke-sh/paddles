# Capability-Negotiated Harness Foundation - Product Requirements

## Problem Statement

Paddles has a differentiated recursive planning harness: typed planner actions, evidence bundles, governance chambers, harness profiles, and trace projections. Compared with open source Codex and opencode, however, the practical coding substrate is not yet mature enough. External capabilities are mostly described but not brokered, execution policy is not expressive enough, editing lacks production safeguards, semantic code intelligence is missing, delegation is not a runtime tool, durable thread state is thin, evals are not established, and the application layer is too monolithic for safe evolution.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Preserve Paddles' recursive planner/synthesizer advantage while closing practical harness gaps against Codex and opencode. | Capability comparison matrix covers external capabilities, execution policy, editing, LSP, delegation, durable sessions, evals, and architecture. | All must-have requirements have voyage and story coverage. |
| GOAL-02 | Make coding operations safer and more observable without weakening local-first constraints. | Governance, permission, edit, shell, and external-capability outcomes are captured as typed events and evidence. | All new tool paths include tests and traceable denial/degraded/success behavior. |
| GOAL-03 | Reshape the core into domain-driven, hexagonal boundaries that can support recursive evolution. | Application orchestration is split behind domain ports and infrastructure adapters. | No new feature deepens the application monolith; refactor slices reduce the largest service surfaces. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | A developer using Paddles as a local-first coding harness. | Reliable code edits, shell execution, semantic navigation, and reviewable evidence. |
| Harness Maintainer | A contributor evolving the recursive runtime. | Clear domain ports, adapter boundaries, tests, and evals that catch regressions. |
| Recursive Planner | The model lane making bounded decisions inside Paddles. | Live capability disclosure, explicit constraints, typed evidence, and enough budget to reason. |

## Scope

### In Scope

- [SCOPE-01] Implement real external capability brokering for web, MCP, and connector-style tools through existing typed capability contracts.
- [SCOPE-02] Add a Codex-grade execution policy layer under Paddles governance, including prefix decisions, denial evidence, and operator-visible posture.
- [SCOPE-03] Upgrade workspace edit hands with safer replacements, file locks, line-ending/BOM preservation, diff evidence, formatter hooks, and diagnostics.
- [SCOPE-04] Add semantic workspace intelligence through LSP-backed navigation and diagnostics as typed planner-accessible actions.
- [SCOPE-05] Turn delegation concepts into bounded runtime workers that inherit governance and return typed evidence for parent integration.
- [SCOPE-06] Introduce durable session, rollout, snapshot, replay, compaction, and rollback foundations aligned with existing trace projections.
- [SCOPE-07] Create a recursive harness evaluation suite that verifies capability disclosure, evidence gathering, tool failure recovery, edit obligations, delegation, context pressure, and architecture boundaries.
- [SCOPE-08] Split the application monolith using domain-driven design and hexagonal architecture so domain policy, application orchestration, and infrastructure adapters are independently testable.
- [SCOPE-09] Improve product/runtime surfaces only where needed to expose the new governance, eval, edit, diagnostic, external provenance, and worker evidence paths.

### Out of Scope

- [SCOPE-10] Replacing Paddles' recursive planner/synthesizer model with a generic tool-call loop.
- [SCOPE-11] Adding mandatory network dependencies or hosted services without a new ADR.
- [SCOPE-12] Rewriting the TUI or web UI wholesale before the runtime contracts are stable.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Broker external capabilities through typed descriptors, governed calls, and evidence-bearing results instead of the noop broker. | GOAL-01, GOAL-02 | must | Recursive planning needs live external facts without bypassing local-first governance. |
| FR-02 | Enforce command and tool execution through an expressive policy engine that supports allow, prompt, deny, and on-failure behaviors. | GOAL-02 | must | Everyday coding safety depends on predictable shell and tool posture. |
| FR-03 | Provide a production-grade edit hand with safe replacement, locking, patch evidence, and diagnostic feedback. | GOAL-01, GOAL-02 | must | Coding harness quality is decided by edit reliability. |
| FR-04 | Expose LSP-backed semantic code intelligence as typed workspace actions. | GOAL-01 | must | Semantic navigation reduces brittle shell probing and improves implementation quality. |
| FR-05 | Provide bounded recursive workers that inherit governance, operate within explicit ownership, and return evidence to the parent loop. | GOAL-01, GOAL-02 | should | Delegation should amplify recursion without becoming unmanaged parallel chat. |
| FR-06 | Persist session, trace, rollout, snapshot, compaction, and rollback state for replayable work. | GOAL-01, GOAL-02 | must | Durable coding sessions need recoverability and auditability. |
| FR-07 | Add a harness eval runner and initial eval corpus covering recursive, tool, edit, delegation, and architecture regressions. | GOAL-03 | must | The architecture cannot evolve safely without repeatable proofs. |
| FR-08 | Decompose application orchestration into domain ports, application services, and infrastructure adapters. | GOAL-03 | must | The current application surface is too large for safe recursive harness evolution. |
| FR-09 | Surface new governance, provenance, diagnostics, worker, and eval outcomes in existing CLI/web/TUI projections. | GOAL-01, GOAL-02 | should | Operators need runtime reality, not hidden controller state. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Preserve local-first operation; external network capabilities remain optional, declared, and governed. | GOAL-02 | must | Local-first is a foundational constraint. |
| NFR-02 | Follow TDD for each slice: failing test, minimum implementation, refactor with tests green. | GOAL-03 | must | The board contract requires evidence-backed delivery. |
| NFR-03 | Keep recursive reasoning model-owned: the harness discloses capabilities and constraints, validates outcomes, and avoids controller-authored pseudo-plans. | GOAL-01 | must | This preserves Paddles' unique recursive capability. |
| NFR-04 | Maintain typed evidence and event projection for every new runtime capability. | GOAL-01, GOAL-02 | must | Traceability is part of the product, not an afterthought. |
| NFR-05 | Refactors must maintain behavior and avoid broad unrelated churn. | GOAL-03 | must | The monolith split must reduce risk rather than create it. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Capability parity | Matrix review against Codex and opencode features | PRD/voyage/story trace plus eval results |
| External capabilities | Unit and integration tests for unavailable, denied, degraded, and success results | Typed evidence assertions and projection events |
| Execution policy | Policy parser/decision tests and shell-hand integration tests | Denial, prompt, allow, and on-failure evidence |
| Editing and LSP | File-format preservation tests, replacement tests, diagnostics tests | Diff evidence, diagnostics output, and passing unit tests |
| Delegation | Parent/worker lifecycle tests with inherited governance and evidence integration | Trace events and parent-loop evidence |
| Durability | Session replay, compaction, snapshot, and rollback tests | Replayable stored state and recovery proofs |
| Architecture | Module boundary tests and compile-level dependency checks where practical | Application surface reduction and domain-port coverage |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Paddles should remain local-first and recursive rather than mimicking Codex or opencode directly. | Work may optimize for the wrong architecture. | Mission constraints and ADR review. |
| The existing domain contracts are strong enough to extend rather than replace. | Refactor scope may expand. | First monolith split and adapter extraction slices. |
| Operators value reliability and auditability over raw tool count. | Product priorities may shift. | Evals and comparison matrix review. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which external capability implementation should ship first: web, MCP, or connector apps? | Mission owner | Open, default to smallest governed web capability. |
| Which session store backend best preserves local-first constraints? | Architecture owner | Open, evaluate file-backed and embedded database options. |
| How strict should policy defaults be for shell and external tools? | Governance owner | Open, start with explicit posture and no hidden network escalation. |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Active voyages and stories cover every must-have functional requirement.
- [ ] The recursive harness eval suite exists and runs locally.
- [ ] External capability, execution policy, edit, LSP, delegation, and durability slices produce typed evidence.
- [ ] Application orchestration is split behind domain ports and infrastructure adapters without weakening recursive behavior.
- [ ] Documentation explains the DDD/hexagonal boundary map and how it preserves recursive capabilities.
<!-- END SUCCESS_CRITERIA -->
