# Normalized Deliberation Signals And Rationale Compilation - Software Design Description

> Normalize provider-native reasoning into harness-safe deliberation signals and use those signals to improve refine, branch, and stop decisions while keeping paddles rationale explicit and auditable.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage converts provider-native reasoning mechanics into something the
harness can safely use. Adapters or application-normalization layers produce a
provider-agnostic `DeliberationSignals` struct, the recursive harness consumes
those signals when deciding whether to continue, branch, refine, retry, or
stop, and paddles still emits its own concise `rationale` as the canonical
explanation of the chosen action.

## Context & Boundaries

- In scope:
  - normalized deliberation signal schema
  - signal extraction from provider-native substrate
  - signal-aware recursive harness decisions
  - rationale compilation and operator-surface boundaries
- Out of scope:
  - replacing paddles rationale with provider-native reasoning
  - exposing raw provider-native reasoning in default transcript surfaces
  - unrelated planner prompt tuning

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│  provider substrate -> signal extractor -> deliberation signals    │
│                                     -> recursive harness policy    │
│                                     -> action + paddles rationale  │
│                                                                    │
│  operator surfaces <---- rationale + signal summary only          │
└────────────────────────────────────────────────────────────────────┘
        ↑                                           ↑
   provider-native state                       canonical turn state
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Deliberation substrate from Voyages 1 and 2 | internal | Source material for normalized signals | planned |
| Recursive harness orchestration | internal | Consumer of provider-agnostic signals | current |
| Read-model/operator surfaces | internal | Present rationale and signal summaries without raw provider-native reasoning | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical signal contract | Define explicit provider-agnostic `DeliberationSignals` in the application layer | Keeps harness policy portable across providers |
| Absence semantics | Represent missing signals explicitly rather than inferring support from nulls or provider names | Prevents subtle control-path bugs |
| Rationale ownership | Compile paddles `rationale` from action, evidence, and normalized signals | Retains a concise, auditable control artifact |
| Operator visibility | Surface signal summaries and evidence, not raw provider-native reasoning | Preserves observability without coupling surfaces to provider payloads |

## Architecture

1. Provider adapters or normalization layers emit zero or more normalized
   `DeliberationSignals`.
2. The recursive harness consumes those signals to influence bounded decisions:
   continue, branch, refine, retry, or stop.
3. The chosen action is recorded with a paddles-owned `rationale` derived from
   action, evidence, and relevant normalized signals.
4. Transcript/manifold/forensic surfaces display rationale and signal summaries
   while leaving raw provider-native reasoning outside canonical output.

## Components

`DeliberationSignals`
: Provider-agnostic schema carrying bounded hints such as uncertainty, missing
evidence, branch candidates, tool-continuation requirement, stop confidence, and
risk flags.

`SignalExtractor`
: Provider-aware translator that turns native reasoning artifacts into
normalized signals or explicit `none`/`unknown`.

`SignalAwareRecursivePolicy`
: Recursive harness policy logic that consumes normalized signals without
matching on provider names.

`RationaleCompiler`
: Produces the final paddles `rationale` from selected action, evidence, and
normalized signals.

`OperatorSignalProjection`
: Read-model or presentation layer that exposes signal summaries and rationale
boundaries without raw provider-native reasoning.

## Interfaces

Candidate internal interfaces:

- `extract_deliberation_signals(provider_state, provider_artifacts) -> DeliberationSignals`
- `apply_signals_to_recursive_policy(signals, current_budget, evidence) -> PolicyHints`
- `compile_rationale(action, evidence, signals) -> String`
- `project_operator_signals(turn) -> SignalSummary`

## Data Flow

1. A provider adapter finishes a turn with native continuation artifacts or a
   deliberate no-op result.
2. A signal extractor converts that substrate into normalized signals.
3. The recursive harness reads those signals before deciding whether to branch,
   refine, continue after tools, retry, or stop.
4. Once the action is selected, the rationale compiler emits a concise paddles
   `rationale`.
5. Operator surfaces read signal summaries and rationale from canonical turn
   state without accessing raw provider-native reasoning.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Provider-native signals are too provider-specific and leak into policy code | Review or tests find provider-name conditionals in harness logic | Block the slice and move translation back into extractors | Keep the signal schema provider-agnostic |
| Signal absence is misinterpreted as low risk or stop confidence | Decision-path tests fail on no-op providers | Represent absence explicitly and make policy conservative by default | Re-run native vs no-op provider tests |
| Rationale starts echoing raw provider reasoning | Review or projection tests detect raw provider content in rationale/surfaces | Remove direct provider text dependencies from the compiler | Rebuild rationale from action/evidence/signal summaries only |
