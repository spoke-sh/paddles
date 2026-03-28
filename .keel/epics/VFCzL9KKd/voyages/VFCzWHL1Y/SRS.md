# Bounded Autonomous Gatherer Integration - SRS

## Summary

Epic: VFCzL9KKd
Goal: Use Sift's autonomous planner as an evidence-first gatherer lane for multi-hop repository investigation while preserving paddles' current synthesizer-first control plane.

## Scope

### In Scope

- [SCOPE-01] Extend the context-gathering contract to return planner-aware metadata alongside ranked evidence.
- [SCOPE-02] Implement a local Sift autonomous gatherer adapter that wraps the supported upstream autonomous planner APIs.
- [SCOPE-03] Route decomposition-worthy prompts through the autonomous gatherer lane before final synthesis.
- [SCOPE-04] Preserve the current synthesizer and deterministic tool paths for prompts that do not need autonomous retrieval planning.
- [SCOPE-05] Surface planner telemetry, retained artifacts, and fallback causes in verbose/debug output.
- [SCOPE-06] Add evaluation or proof coverage comparing static context assembly and autonomous retrieval planning.

### Out of Scope

- [SCOPE-07] Replacing the synthesizer lane with autonomous planning.
- [SCOPE-08] Implementing branching or graph-structured planning beyond Sift's current bounded linear runtime.
- [SCOPE-09] Making model-driven planner profiles mandatory for local operation.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The typed gatherer result must carry synthesis-ready evidence plus planner strategy, planner trace, planner stop reason, retained artifacts, and warnings without pretending to be the final answer. | SCOPE-01 | FR-01 | manual |
| SRS-02 | A local Sift autonomous gatherer adapter must map `ContextGatherRequest` into `Sift::search_autonomous` and return stable evidence-first results for the synthesizer lane. | SCOPE-02 | FR-02 | manual |
| SRS-03 | The controller must classify decomposition-worthy prompts and route them through the autonomous gatherer lane before final synthesis. | SCOPE-03 | FR-03 | manual |
| SRS-04 | Ordinary chat, coding, deterministic tool turns, and shallow retrieval must remain on the current synthesizer-first path when autonomous planning is not selected or not available. | SCOPE-04 | FR-04 | manual |
| SRS-05 | Verbose/debug output must expose planner strategy, step count or trace summary, stop reason, retained artifacts, and fallback causes for autonomous-gatherer turns. | SCOPE-05 | FR-05 | manual |
| SRS-06 | The repository must include proof or evaluation coverage comparing static context assembly and autonomous retrieval planning on representative investigation prompts. | SCOPE-06 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The default answer/tool path must remain local-first with no new mandatory remote dependency for ordinary prompt handling. | SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | Autonomous gatherer failures must degrade safely to the existing synthesizer path without corrupting routing state or breaking deterministic tool execution. | SCOPE-02, SCOPE-03, SCOPE-04 | NFR-02 | manual |
| SRS-NFR-03 | Planner telemetry must be observable enough for operators to debug why autonomous planning was chosen, skipped, or aborted. | SCOPE-05 | NFR-03 | manual |
| SRS-NFR-04 | Planner-strategy support must remain extensible so heuristic and model-driven planners can be selected without duplicating controller logic. | SCOPE-01, SCOPE-02 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
