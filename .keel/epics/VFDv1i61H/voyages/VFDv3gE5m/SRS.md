# Recursive Planner Harness Backbone - SRS

## Summary

Epic: VFDv1i61H
Goal: Replace static turn heuristics with a model-owned bounded search/refine loop that interprets operator memory first, gathers recursive evidence, and hands structured trace plus evidence to a downstream synthesizer.

## Scope

### In Scope

- [SCOPE-01] Use operator memory and linked foundational docs as first-pass interpretation context for planner-capable turns.
- [SCOPE-02] Replace coarse static turn classification as the main reasoning mechanism with planner action selection behind a bounded contract.
- [SCOPE-03] Add a recursive resource loop where the planner can search, read, inspect tool output, refine, branch, and stop.
- [SCOPE-04] Keep planner and synthesizer roles distinct so final answers are synthesized from recursive evidence.
- [SCOPE-05] Update foundational docs and diagrams so the recursive harness is the documented backbone architecture.

### Out of Scope

- [SCOPE-06] Hardcoded Keel-specific runtime intents or bespoke board-only controllers.
- [SCOPE-07] Mandatory remote planners or removal of local-first fallbacks.
- [SCOPE-08] Replacing the interactive TUI or unrelated boot/pacemaker mechanics.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Planner-capable turns must assemble interpretation context from operator memory, recent turns, and relevant foundational guidance before choosing the next resource action. | SCOPE-01 | FR-01 | manual |
| SRS-02 | The planner boundary must expose a constrained action contract that can express search, read, inspect, refine, branch, and stop decisions. | SCOPE-02, SCOPE-03 | FR-02 | manual |
| SRS-03 | The recursive planner loop must execute multiple bounded resource steps until it either reaches a stop condition, exhausts budget, or produces synthesis-ready evidence. | SCOPE-03 | FR-02 | manual |
| SRS-04 | The final answer path must remain a distinct synthesizer step that consumes planner trace and evidence rather than trusting planner prose as the answer. | SCOPE-04 | FR-04 | manual |
| SRS-05 | Model routing must allow planner and synthesizer roles to target different configured models or providers according to runtime constraints. | SCOPE-04 | FR-05 | manual |
| SRS-06 | Foundational docs must describe the recursive harness backbone with diagrams for interpretation context, recursive loop behavior, and planner/synth routing. | SCOPE-05 | FR-06 | manual |
| SRS-07 | The design must avoid making Keel board state a first-class runtime intent; board knowledge should flow through the same recursive context mechanisms as other workspace evidence. | SCOPE-01, SCOPE-02, SCOPE-03 | FR-07 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Recursive planning must remain local-first by default and degrade safely when a heavier planner provider is unavailable. | SCOPE-01, SCOPE-03, SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | The recursive loop must honor explicit action, depth, and evidence budgets so it cannot spin indefinitely. | SCOPE-02, SCOPE-03 | NFR-02 | manual |
| SRS-NFR-03 | Planner traces, action decisions, stop reasons, and synthesizer handoff data must remain observable to operators. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-03 | manual |
| SRS-NFR-04 | The recursive harness contract must remain general-purpose across repositories and evidence domains rather than Keel-specific. | SCOPE-01, SCOPE-02, SCOPE-03, SCOPE-04 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
