# Deliberation Substrate And Continuation Contracts - Software Design Description

> Establish provider-agnostic deliberation state contracts and adapter continuity boundaries so provider-native reasoning can be carried between steps without becoming canonical paddles rationale.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces the substrate that all later reasoning work depends on.
Adapters gain an explicit deliberation capability and an opaque continuation
state that can survive recursive tool/result turns. The harness keeps using
provider-agnostic control artifacts, while supported adapters can privately
carry native reasoning state forward. Moonshot/Kimi is the proving path for the
first real continuity implementation, and unsupported providers must still
advertise an explicit no-op capability.

## Context & Boundaries

- In scope:
  - capability negotiation for provider reasoning support
  - opaque adapter-owned continuation state
  - Moonshot/Kimi continuity support
  - debug-scoped reasoning artifact boundaries
  - native/no-op contract tests
- Out of scope:
  - recursive planner signal consumption
  - full provider rollout beyond the proving path
  - user-facing display of raw provider reasoning

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│  provider + model -> deliberation capability -> adapter session    │
│                                 -> opaque continuation state        │
│                                 -> next provider call              │
│                                                                    │
│  debug traces (optional) <---- provider artifact recorder          │
└────────────────────────────────────────────────────────────────────┘
        ↑                                           ↑
  native reasoning payloads                 canonical turn state
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `ModelCapabilitySurface` negotiation | internal | Existing place to extend provider/model runtime capabilities | current |
| HTTP provider adapters | internal | Carry provider-native reasoning continuity where supported | current |
| Trace/forensic recorder | internal | Optional bounded storage for debug-scoped reasoning artifacts | current |
| Moonshot/Kimi reasoning transport | external API | First native continuity proving path | current provider contract |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical control artifact | Keep paddles `rationale` separate from provider-native reasoning | Provider reasoning is useful for execution continuity, not as the canonical operator-facing explanation |
| State shape | Store provider-native reasoning in opaque `DeliberationState` | Lets adapters evolve per-provider payloads without leaking them into the rest of the runtime |
| First implementation | Prove the substrate with Moonshot/Kimi continuity first | Kimi explicitly requires continuity across multi-step thinking/tool turns and gives a concrete target for the contract |
| Unsupported behavior | Represent unsupported providers explicitly instead of implicit `None` | Safe degradation is part of the runtime contract |

## Architecture

1. Capability negotiation extends the provider/model surface with a
   deliberation-support classification.
2. Adapter calls may return an updated opaque `DeliberationState` alongside
   their normal authored-response output.
3. Supported providers feed the stored continuation state back on the next
   provider call when the recursive harness continues after tools/results.
4. Optional provider-native reasoning artifacts are recorded only on
   debug/forensic paths, never in canonical transcript/render truth.
5. Contract tests verify both a native continuity provider path and an explicit
   no-op path.

## Components

`DeliberationCapabilitySurface`
: Extends provider/model negotiation with explicit support levels for native
reasoning continuity and fallback behavior.

`DeliberationState`
: Opaque provider-scoped continuation artifact that can be attached to adapter
sessions or turn execution context without entering domain state.

`ProviderContinuationBridge`
: Adapter-local logic that replays provider-native reasoning substrate on the
next request for supported providers.

`ReasoningArtifactRecorder`
: Debug-scoped recorder for bounded provider-native reasoning artifacts when the
runtime needs forensic visibility.

`MoonshotReasoningBridge`
: First native provider implementation that carries Kimi continuity across
tool/result turns.

## Interfaces

Candidate internal interfaces:

- `capability_surface(provider, model_id) -> ModelCapabilitySurface`
- `adapter_turn(...) -> { authored_response, deliberation_state, provider_artifacts }`
- `continue_turn(previous_deliberation_state, tool_results, ...)`
- `record_debug_reasoning_artifact(task_id, provider_artifact, ...)`

## Data Flow

1. The runtime negotiates a provider/model capability surface before a turn.
2. The adapter executes the provider call and may return:
   - normal authored output
   - updated opaque `DeliberationState`
   - optional provider-native reasoning artifacts for debug use
3. If the harness continues after tool execution, the adapter receives the
   stored `DeliberationState` and translates it back into the provider-specific
   continuation format.
4. Canonical transcript/render persistence continues to use only authored
   response and paddles-owned control artifacts.
5. Contract tests assert the supported provider path reuses continuity state and
   the unsupported provider path exposes a deliberate no-op classification.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Provider advertises reasoning continuity but returns unusable continuation state | Adapter contract tests or runtime validation fail | Downgrade to explicit limited/no-op behavior and surface the mismatch in debug traces | Repair the provider adapter contract before expanding rollout |
| Raw provider reasoning leaks into canonical transcript/render state | Review/test catches provider fields in persisted turn state | Block the slice and remove the leakage path | Keep provider artifacts on debug-only storage paths |
| Debug reasoning artifacts grow unbounded | Recorder tests or manual review show oversized retention | Clamp or redact the stored artifact | Preserve only bounded forensic summaries or opaque handles |
