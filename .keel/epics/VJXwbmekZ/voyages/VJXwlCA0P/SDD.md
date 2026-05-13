# Unified Agent Action Contract - Software Design Description

> Define one recursive agent action decision contract that covers first and subsequent model choices without a separate InitialAction routing type.

**SRS:** [SRS.md](SRS.md)

## Overview

Introduce one domain contract for model-selected bounded actions. The first
model decision and later recursive decisions should share the same action type
and decision envelope; differences such as whether `answer` is currently
available belong to schema variants and loop state, not separate Rust action
universes.

## Context & Boundaries

This voyage changes contract shape and tests. It does not yet move runtime
control flow. The output should make the runtime migration possible in a
behavior-preserving follow-up.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing `InitialAction`, `PlannerAction`, and `WorkspaceAction` | Rust domain types | Source behavior to preserve during migration | current repo |
| Shared planner action schema renderer | Application contract | Prompt-visible action schema source | current repo |
| Sift and HTTP parser tests | Adapter tests | Prove JSON envelope parity | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Contract shape | One agent action enum plus one decision envelope | The model's reasoning is bounded action selection in the loop. |
| Variant behavior | Use schema variants for availability, not separate action universes | First and later prompts can differ without duplicating contracts. |
| Compatibility | Allow temporary aliases only with tests | Keeps migration safe while preventing permanent double vocabulary. |

## Architecture

The intended contract shape is:

```rust
pub enum AgentAction {
    Answer { answer: String },
    Workspace { action: WorkspaceAction },
    Refine { query: String, mode: RetrievalMode, strategy: RetrievalStrategy, retrievers: Vec<RetrieverOption> },
    Branch { branches: Vec<String> },
    Stop { reason: String, answer: Option<String> },
}

pub struct AgentDecision {
    pub action: AgentAction,
    pub rationale: String,
    pub edit: InitialEditInstruction,
    pub grounding: Option<GroundingRequirement>,
    pub deliberation_state: Option<DeliberationState>,
}
```

The exact names can change during implementation, but the shape must keep
terminal actions, workspace actions, refinement, branch, edit metadata, and
grounding in one envelope.

## Components

| Component | Purpose |
|-----------|---------|
| Domain action contract | Owns the bounded action vocabulary |
| Shared schema renderer | Renders prompt-visible JSON examples from the same vocabulary |
| Contract parity tests | Prove Rust enum labels and schema action names do not drift |

## Interfaces

No external API contract changes are required in this voyage. Adapter-specific
JSON parsing may keep compatibility wrappers while tests pin the unified target
contract.

## Data Flow

1. Domain exposes one action vocabulary.
2. Schema renderer reads the same action vocabulary.
3. First and later schema variants filter availability rather than owning
   separate action lists.
4. Parser tests prove semantic and external actions map into the same domain
   contract.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Schema contains an action not in the Rust contract | Parity test failure | Fail tests | Update schema or enum |
| Old compatibility name becomes a second contract | Compatibility test failure or grep proof | Fail story | Remove alias or document bounded transition |
| First/later variants drift | Schema renderer test failure | Fail tests | Reuse canonical entries |
