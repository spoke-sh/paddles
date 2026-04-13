# Establish A Typed Collaboration Mode And Review Substrate - Software Design Description

> Establish typed collaboration mode state, a findings-first review lane, and bounded structured clarification on top of the recursive harness.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage adds typed collaboration-mode semantics to the existing recursive
harness so planning, execution, and review become explicit runtime behaviors
instead of prompt suggestions. The same controller remains in charge, but it
receives a typed mode selection, enforces mode-specific mutation posture,
supports bounded structured clarification during planning, and normalizes
review-mode output into a findings-first contract.

The slice does not create separate planners or products. It layers a mode-aware
policy and projection substrate around the current harness, then exposes those
mode transitions and structured question exchanges through the existing trace,
transcript, and UI paths.

## Context & Boundaries

In scope are:
- typed runtime state for planning, execution, and review modes
- mode-specific prompt, permission, and output contracts
- bounded structured clarification request/response flows for planning
- a dedicated findings-first review workflow over local changes
- projection of mode state, clarification pauses, and review findings across
  trace, transcript, UI, API, and docs

Out of scope are:
- style-only personas that do not alter workflow semantics
- hosted PR or ticketing backplanes
- multi-agent delegation behavior beyond preserving future compatibility

```
┌────────────────────────────────────────────────────────────┐
│     This Voyage: Mode-Aware Planning And Review Layer     │
│                                                            │
│ Surface Intent -> Collaboration Mode State -> Harness      │
│                                   ↓                        │
│       Planning Clarification + Review Output Contracts     │
│                                   ↓                        │
│          Governance / Trace / Transcript / UI Projection   │
└────────────────────────────────────────────────────────────┘
           ↑                          ↑
      Local diff/worktree        User clarification
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing recursive planner and runtime loop | Internal runtime | Remain the single orchestration loop beneath all collaboration modes | current repo |
| Execution governance substrate from mission VGb1Xq72Y | Internal runtime | Apply fail-closed mutation posture and escalation semantics per mode | current repo |
| Replayable control/runtime-item substrate from mission VGb1YwKRk | Internal runtime | Carry mode transitions, clarification exchanges, and findings through shared projections | current repo |
| Local diff and workspace inspection paths | Internal runtime | Supply grounded review evidence without requiring hosted review systems | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Mode model | Use one typed collaboration-mode contract for planning, execution, and review | The harness should steer behavior structurally without fragmenting into separate products |
| Planning clarification | Support bounded structured question/answer exchanges instead of ad hoc conversational probing | Clarification should be explicit, typed, and replayable |
| Review output | Make findings-first review a workflow contract rather than a prompt convention | Review quality depends on enforcing a stable output shape |
| Mutation posture | Preserve execution as the default mutation mode while planning and review fail closed on mutation | Mode semantics must affect safety posture, not just prose |
| Projection | Surface mode state and structured clarification through existing trace/transcript/UI adapters | Operators should not need a separate observability path for mode behavior |

## Architecture

The voyage adds a mode-aware substrate around the recursive harness:

1. A typed collaboration-mode state is resolved for the turn or session.
2. The harness derives prompt, permission, and output expectations from that
   selected mode.
3. Planning mode can pause into a bounded structured clarification exchange
   when the runtime genuinely needs user input.
4. Review mode inspects local changes through the existing evidence-first
   controller and emits findings-first results.
5. Trace, transcript, TUI, web, and API projections replay mode changes,
   clarification exchanges, and findings through one shared vocabulary.

## Components

- `CollaborationModeState`
  Purpose: represent planning, execution, and review as typed runtime state.
  Interface: mode selection, mode request, and degrade-result contracts.
  Behavior: shapes prompting, permission posture, and output expectations.

- `StructuredClarificationContract`
  Purpose: model bounded user-input requests and responses.
  Interface: typed question envelope plus typed answer payload.
  Behavior: pauses planning deliberately instead of falling back to vague chat.

- `ReviewWorkflowContract`
  Purpose: define findings-first review semantics.
  Interface: grounded finding records, residual-risk summaries, and review-mode
  final response contract.
  Behavior: ensures review output is ordered by findings and anchored to local
  evidence.

- `ModeAwareGovernanceBridge`
  Purpose: compose collaboration mode with mutation permissions.
  Interface: mode-resolved permission posture and fail-closed denial results.
  Behavior: planning and review restrict mutation structurally, while execution
  remains the default mutation lane.

- `ModeProjectionAdapters`
  Purpose: render mode changes, clarification exchanges, and review findings on
  shared operator surfaces.
  Interface: trace, transcript, runtime-event, and UI projection sinks.
  Behavior: keeps one readable vocabulary across TUI, web, and API.

## Interfaces

- `select_collaboration_mode(intent) -> CollaborationModeResult`
- `request_structured_clarification(prompt) -> StructuredQuestion`
- `record_structured_clarification_answer(answer) -> ClarificationOutcome`
- `run_review_workflow(scope) -> ReviewFindingBundle`
- `project_mode_runtime_state(item) -> ProjectionEvent`

## Data Flow

1. A turn or session selects planning, execution, or review mode.
2. The runtime derives mode-specific prompt, permission, and output posture.
3. Planning mode either continues non-mutating exploration or emits a bounded
   structured clarification request.
4. Execution mode remains the default mutation path under normal governance.
5. Review mode inspects local changes through the existing evidence loop and
   produces findings-first output.
6. Trace and projection layers record mode changes, clarification exchanges,
   and review findings for replay and live operator surfaces.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Unknown or unsupported mode request | Mode selection cannot resolve a valid collaboration mode | Emit a typed degraded mode result instead of silently reverting to execution | Operator can choose a supported mode and retry |
| Planning mode needs clarification but structured requests are unavailable | Clarification boundary cannot open the bounded request path | Emit an explicit unavailable clarification result and stop or degrade honestly | Operator can answer later or rerun in execution mode |
| Review scope lacks local evidence | Diff/worktree inspection yields no actionable review material | Emit no-findings-with-risks output instead of fabricating findings | Expand review scope or make local changes visible |
| Planning or review attempts mutation | Mode-aware governance detects a blocked mutating action | Fail closed with a mode-specific denial result | Switch to execution mode or explicitly resume after review |
| Surface cannot custom-render a mode or clarification item | Projection receives an unfamiliar item kind | Preserve the item in trace and show a generic readable summary | Extend the surface adapter without changing the underlying contract |
