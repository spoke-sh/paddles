# Rename MechSuitService And Chambers To Idiomatic Modules - Software Design Description

> Mechanical, behavior-preserving renames: MechSuitService -> AgentRuntime; *Chamber -> plain function modules (agent_loop, context_assembly, synthesis, turn); RecursiveControlChamber -> agent_loop. Land each rename as its own reviewable diff. Subsequent voyages handle ExecutionHand, WorkspaceAction, specialist_brains, harness_profile, gatherer, forensics, and the steering/deliberation/compaction/premise term sweeps.

**SRS:** [SRS.md](SRS.md)

## Overview

`src/application/mod.rs` is a 17,556-line monolith built around a single `MechSuitService` god-object. Phase logic is organized into "chambers" — `RecursiveControlChamber`, `InterpretationChamber`, `SynthesisChamber`, `TurnOrchestrationChamber`, and so on — each defined as `pub(super) struct Chamber<'a> { service: &'a MechSuitService }` with methods that delegate every meaningful call back to the service. The chambers hold no state and provide no isolation; they're function namespaces wearing a noun. New contributors (and the author on a tired Friday) burn calories mapping bespoke vocabulary to operations the rest of the agent-tooling field already names.

This voyage performs three mechanical, behavior-preserving renames as separately reviewable diffs:

1. **`MechSuitService` → `AgentRuntime`.** Rename the struct, all impl blocks, the factory closure types (`SynthesizerFactory`, `PlannerFactory`, `GathererFactory` ownership remains the same), every `Arc<MechSuitService>` site, and every test mention. Trace artifact identifiers and on-disk record schemas are unchanged — only the in-process type name moves.
2. **`*Chamber` types → plain function modules.** Each chamber wrapper is deleted. Its methods become free functions in a module named for the phase: `RecursiveControlChamber::execute_recursive_planner_loop` → `agent_loop::execute(...)`; `InterpretationChamber::derive_interpretation_context` → `context_assembly::derive(...)`; `SynthesisChamber::*` → `synthesis::*`; `TurnOrchestrationChamber::*` → `turn::*`.
3. **`recursive_control` module → `agent_loop`.** The file is renamed and `mod` declarations and `use` paths updated.

Each rename lands as its own git commit with no behavior change — easy to review, easy to revert, easy to bisect. CLI flags, web HTTP routes, persisted trace record schemas, and provider/governance behavior are all out of scope and untouched. This voyage explicitly does **not** split `mod.rs` into separately-tested services; sibling voyages under epic VI2sJZcV9 will rename the next tier (`ExecutionHand` → `ToolRunner`, `WorkspaceAction` → `Tool`, `specialist_brains` → `subagents`, `gatherer` → `retriever`, `forensics` → `trace`/`inspector`, `harness_profile` → `runtime_profile`, and the steering / deliberation / compaction / premise term sweeps).

## Components

- `src/application/mod.rs` — `MechSuitService` → `AgentRuntime`; chamber wrappers deleted; methods migrated to module-level functions.
- `src/application/recursive_control.rs` → `src/application/agent_loop.rs`.
- `src/application/interpretation_chamber.rs` → `context_assembly.rs`; `synthesis_chamber.rs` → `synthesis.rs`; `turn_orchestration.rs` → `turn.rs`.
- `src/main.rs`, `src/lib.rs`, `src/infrastructure/web/mod.rs`, `src/infrastructure/cli/interactive_tui.rs`, and `tests/` — every `MechSuitService` and `*Chamber` reference updated.
- Verification: `cargo check`, `cargo test`, `keel doctor`, plus `git grep -E '\b(MechSuitService|RecursiveControlChamber|InterpretationChamber|SynthesisChamber|TurnOrchestrationChamber)\b'` returns no hits in `src/`, `tests/`, or `apps/`.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

<!-- Component relationships, layers, modules -->

## Components

<!-- For each major component: purpose, interface, behavior -->

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
