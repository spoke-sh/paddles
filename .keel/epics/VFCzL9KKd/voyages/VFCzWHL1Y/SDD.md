# Bounded Autonomous Gatherer Integration - Software Design Description

> Use Sift's autonomous planner as an evidence-first gatherer lane for multi-hop repository investigation while preserving paddles' current synthesizer-first control plane.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage extends the existing gatherer/synthesizer split instead of creating
a second answer stack. The controller keeps ownership of routing. When a prompt
looks decomposition-worthy, it invokes a new local Sift autonomous gatherer
adapter. That adapter performs bounded iterative retrieval through Sift's
supported autonomous planner runtime, then returns a typed evidence-first result
to the existing synthesizer lane.

The implementation goal is architectural honesty. Autonomous retrieval planning
should gather evidence and planner metadata, not bypass the synthesizer or hide
routing decisions in prompt text.

## Context & Boundaries

This design covers planner-aware gatherer contracts, a local Sift autonomous
gatherer adapter, controller routing updates, verbose telemetry, and comparison
proofs against the existing static context-assembly path.

It does not cover replacing the synthesizer lane, introducing graph or branch
planning, or making a model-driven planner profile mandatory for local
operation.

```text
User Prompt
    |
    v
Controller Heuristics
    | ordinary/chat/tool
    +----------------------------> Synthesizer lane
    |
    | decomposition-worthy
    v
Sift Autonomous Gatherer
    |
    v
Evidence Bundle + Planner Metadata
    |
    v
Synthesizer lane
    |
    v
Final Response
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `sift` | Rust crate | Provides `search_autonomous`, autonomous request/response DTOs, and planner strategy selection | Git `main` |
| Existing `ContextGatherer` port | Internal interface | Keeps autonomous planning behind the current gatherer/synthesizer split | Current repo head |
| Existing `SiftAgentAdapter` synthesizer path | Internal runtime | Consumes gathered evidence and continues to own final answer generation | Current repo head |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Gatherer output shape | Extend the typed gatherer result instead of returning planner prose | Preserves the evidence-first contract and keeps final synthesis separate |
| Default planner strategy | Heuristic local planner first | Works with the shipped Sift surface and keeps model-driven planning optional |
| Routing authority | Keep routing in `MechSuitService` controller heuristics | Avoids hiding lane selection in prompt text |
| Failure behavior | Fail closed to the synthesizer path with explicit logs | Preserves the current common path and keeps operator trust |

## Architecture

The design touches three layers:

- `src/domain/ports/context_gathering.rs` for planner-aware gatherer contracts
- a new autonomous gatherer adapter in `src/infrastructure/adapters`
- `src/application/mod.rs` for prompt classification, lane selection, and
  telemetry/fallback behavior

## Components

- `ContextGatherRequest` / `ContextGatherResult`
  Extended to represent planner metadata, retained artifacts, and warnings
  alongside ranked evidence.
- `SiftAutonomousGathererAdapter`
  Calls `Sift::search_autonomous`, maps the planner response into the gatherer
  contract, and enforces local capability and failure behavior.
- Controller routing
  Selects autonomous planning only for decomposition-worthy prompts and falls
  back cleanly when unsupported, failed, or unnecessary.
- Evaluation/proof harness
  Compares autonomous retrieval planning with static context assembly on
  representative repository-investigation prompts.

## Interfaces

- `ContextGatherRequest`
  Carries the user query, workspace root, budget, and prior context for
  planner-oriented retrieval.
- `ContextGatherResult`
  Carries synthesis-ready evidence, planner metadata, retained artifacts,
  warnings, and explicit capability/fallback semantics.
- `Sift::search_autonomous`
  The adapter invokes the supported upstream API with a planner strategy and
  workspace root.

## Data Flow

1. The controller receives a prompt.
2. Routing heuristics decide whether the prompt is decomposition-worthy.
3. If not, the prompt stays on the current synthesizer path.
4. If yes, the controller builds a `ContextGatherRequest` and sends it to the
   Sift autonomous gatherer adapter.
5. The adapter invokes `Sift::search_autonomous`, receives planner output, and
   converts it into a typed evidence bundle plus planner metadata.
6. The controller logs planner telemetry in verbose mode and forwards the
   evidence bundle into the synthesizer lane.
7. If the adapter fails or is unavailable, the controller logs the failure
   reason and falls back to the ordinary synthesizer path.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Prompt misclassified as decomposition-worthy | Routing tests or verbose traces show unnecessary planner use | Tighten heuristics or add explicit operator control later | Keep synthesizer fallback available |
| Sift autonomous planning fails | Adapter returns error or unsupported state | Log planner failure and skip evidence bundle injection | Fall back to synthesizer-only response |
| Planner metadata exceeds evidence budget | Evidence trimming or telemetry review shows oversized payloads | Trim summaries/snippets and carry warnings forward | Preserve ranked evidence first |
| Model-driven planner profile unavailable | Capability gate or config lookup fails | Downgrade to heuristic strategy with explicit messaging | Maintain local-first default behavior |
