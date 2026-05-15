# Retire Pre-Loop Bootstraps And Vocabulary - Software Design Description

> Controller bootstraps, traces, docs, and runtime vocabulary reflect a single agent-loop action-selection architecture with compatibility shims isolated.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage finishes the migration by removing the controller-authored bootstraps and leftover vocabulary that make the old pre-loop path feel authoritative. The loop can still receive edit, commit, review, and grounding signals, but those signals should shape the loop request instead of overriding the first action outside the loop.

## Context & Boundaries

The voyage covers cleanup after the entry-point and turn-contract migrations. It is not a separate architecture change; it removes old scaffolding, aligns traces/docs, and proves that the original stuck-read pattern is not preserved by the new flow.

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| runtime presentation | infrastructure | Shows selected actions, fallbacks, and boundaries. | internal |
| architecture docs | documentation | Owns user/developer-facing turn-loop architecture. | internal |
| loop tests | test suite | Prevents recurrence of pre-loop repeated-read behavior. | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Bootstraps | Convert to loop signals or delete. | Forced first actions undermine model-owned loop reasoning. |
| Vocabulary | Keep old names only where compatibility requires them. | Internal code should describe the current architecture. |
| Regression proof | Add an operator-contract/simple-question scenario. | The triggering user concern should be locked into tests. |

## Architecture

The cleanup should leave `turn.rs` as orchestration, `agent_loop.rs` as the action-selection owner, and runtime presentation as a projection of loop events. Any helper that previously returned an `InitialActionDecision` should either disappear or produce loop context. Static searches should be part of acceptance.

## Components

- Bootstrap cleanup: remove commit, edit, grounding, and review functions that return forced initial actions.
- Trace cleanup: rename or remove event summaries that imply a pre-loop planner action.
- Docs cleanup: update architecture/configuration docs with the single-loop flow.
- Regression tests: exercise simple evidence-backed questions through the unified loop.

## Interfaces

No new public interface is intended. Existing user-facing output should become clearer because there is no separate pre-loop action record.

## Data Flow

1. Turn request enters the agent loop.
2. Loop receives signals for edit, commit, grounding, or review posture.
3. Loop selects a bounded action or answer.
4. Runtime presentation reports loop-selected action and any hard boundary.
5. Final renderer produces the answer or completion summary.

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Static search finds runtime pre-loop symbols | acceptance check | Keep the story open. | Rename/remove or document compatibility-only surface. |
| Regression scenario still repeats evidence reads | focused test | Keep loop review active. | Adjust observation/loop-state feedback rather than adding a hard repeat guard. |
