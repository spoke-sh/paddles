# Rename MechSuitService And Chambers To Idiomatic Modules - SRS

## Summary

Epic: VI2sJZcV9
Goal: Mechanical, behavior-preserving renames: MechSuitService -> AgentRuntime; *Chamber -> plain function modules (agent_loop, context_assembly, synthesis, turn); RecursiveControlChamber -> agent_loop. Land each rename as its own reviewable diff. Subsequent voyages handle ExecutionHand, WorkspaceAction, specialist_brains, harness_profile, gatherer, forensics, and the steering/deliberation/compaction/premise term sweeps.

## Scope

### In Scope

- [SCOPE-01] Rename the `MechSuitService` struct (and `mech_suit*` references) to `AgentRuntime` across `src/`, `tests/`, and trace artifact identifiers.
- [SCOPE-02] Rename the `*Chamber` types (`RecursiveControlChamber`, `InterpretationChamber`, `SynthesisChamber`, `TurnOrchestrationChamber`, and any siblings) to plain function modules: `agent_loop`, `context_assembly`, `synthesis`, `turn`. The wrapper structs are deleted; their methods move to module-level functions.
- [SCOPE-03] Rename the `recursive_control` module to `agent_loop` and update all `use` paths.

### Out of Scope

- [SCOPE-04] Renames of `ExecutionHand`, `WorkspaceAction`, `PlannerAction`, `instruction_frame`, `specialist_brains`, `harness_profile`, `gatherer`, `forensics`, `compaction_cue`, `premise_challenge`, `deliberation_signals`, `steering_signals` (handled by sibling voyages under epic VI2sJZcV9 in subsequent slices).
- [SCOPE-05] Splitting `src/application/mod.rs` (17,556 lines) into separately-tested services. The chamber rename only flattens names; the file split is a separate mission.
- [SCOPE-06] Behavior changes — this voyage is mechanically rename-only.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | After this voyage, `MechSuitService` is renamed to `AgentRuntime`, every `*Chamber` wrapper is replaced by plain function modules with the names listed in SCOPE-02, and `recursive_control` is renamed to `agent_loop`, with `cargo check`, `cargo test`, and `keel doctor` all green. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-01 | automated |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The rename must land as separately reviewable diffs (one per top-level rename) and must not change runtime behavior, public CLI flags, on-disk trace schemas, or web API routes. | SCOPE-01, SCOPE-02, SCOPE-03 | NFR-01 | automated + manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
