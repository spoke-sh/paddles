# Default Gatherer, Grounded Answers, And Action Stream - Software Design Description

> Make repository questions run through explicit evidence gathering by default, require cited synthesis from that evidence, and render a Codex-style action stream as the default REPL experience.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage turns interactive repository questions into an explicit
`classify -> gather -> synthesize -> render` pipeline.

The key design change is to stop treating repo-question retrieval as implicit
private work inside the synthesizer. Instead, the controller should make the
retrieval decision explicitly, gather evidence through the configured gatherer
boundary, pass that bundle into synthesis, and render the same turn steps to
the operator through a default Codex-style event stream.

## Context & Boundaries

### In Scope
- Default gatherer-first handling for repository questions.
- Stronger intent classification and explicit fallback semantics.
- Grounded synthesis with default file citations and insufficient-evidence
  handling.
- A default REPL event stream for classification, gatherer work, tool calls,
  planner steps, synthesis, and fallback.

### Out of Scope
- Replacing the default synthesizer model family.
- Shipping a full-screen TUI.
- Quiet mode or a silent default REPL.

```
┌───────────────────────────────────────────────────────────────┐
│                           This Voyage                        │
│                                                               │
│  Prompt -> Intent Classifier -> Gatherer Selector             │
│                      |                    |                   │
│                      |                Context Gatherer        │
│                      |                    |                   │
│                      +------------> Evidence Bundle ----------+│
│                                       |                       │
│                                 Grounded Synthesizer          │
│                                       |                       │
│                               Turn Event Renderer             │
└───────────────────────────────────────────────────────────────┘
          ↑                              ↑
      Operator REPL                 Sift / Local Models
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `sift` | library | Static context assembly, autonomous planning, and workspace search primitives | current pinned Cargo dependency |
| local Qwen runtime | internal runtime | Final synthesis for chat, actions, and evidence-backed answers | existing candle-backed runtime |
| Keel docs/state | project context | Mission and proof artifacts for planning and verification | current repo board state |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Repository questions use an explicit gatherer boundary by default | Prefer the configured gatherer lane instead of hidden synthesizer-private retrieval | Makes retrieval behavior understandable and improvable |
| Grounded synthesis is citation-first | Final repo answers cite files by default and admit insufficient evidence | Small local models need a stricter contract |
| The action stream is the default UX | Render Codex-style turn events without a quiet flag | Operator trust depends on always-visible execution details |
| Turn telemetry should be typed | Use renderer-facing event structs/enums rather than ad hoc `println!` strings | Keeps behavior reusable across lanes and providers |

## Architecture

The voyage should introduce or strengthen four cooperating layers:

1. Controller classification and routing in the application layer.
2. Explicit gatherer execution and gatherer fallback semantics.
3. Grounded synthesis that consumes evidence bundles and produces cited
   responses.
4. A renderer that turns typed execution events into the default REPL stream.

## Components

- `TurnIntentClassifier`
  Purpose: classify prompts into casual, action, repository question, or
  decomposition/research.
  Behavior: emits a classification event before any gatherer/tool/synthesis
  work begins.

- `ContextGatherCoordinator`
  Purpose: own explicit gatherer selection and evidence/fallback handling for
  repository questions.
  Behavior: invokes the configured gatherer, records success/fallback events,
  and passes evidence bundles forward.

- `GroundedSynthesisContract`
  Purpose: constrain the synthesizer to answer from evidence and cite files.
  Behavior: produces a final answer plus citation metadata, or an explicit
  insufficient-evidence answer.

- `TurnEventRenderer`
  Purpose: render the default Codex-style action stream shown in the REPL.
  Behavior: groups events into concise operator-facing lines with bounded
  detail and truncation rules.

## Interfaces

- `TurnExecutionEvent`
  A typed event family for classification, gatherer selection, planner step,
  tool invocation, tool result, fallback, and final synthesis.

- `GroundedAnswer`
  A structured synthesis result containing answer text, cited files, and an
  `insufficient_evidence` flag or reason.

- `EvidenceBundle`
  Existing gatherer payload; this voyage should strengthen how it is required
  and consumed rather than letting it remain optional for repo-question turns.

## Data Flow

1. Read the prompt from the REPL.
2. Classify the turn intent and emit a classification event.
3. If the turn is a repository question, select the gatherer lane and emit a
   gatherer-selection event.
4. Execute gatherer work, emit gatherer/planner events, and produce an
   evidence bundle or an explicit fallback event.
5. Invoke synthesis with the evidence bundle and grounded-answer contract.
6. Render cited answer output and the full default action stream.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Gatherer unavailable | Capability or config check | Emit fallback event and degrade to explicit synthesizer path | Preserve deterministic action/chat handling |
| Gatherer failure | Adapter error | Emit failure + fallback event | Continue through a clearly labeled fallback path |
| Weak or missing evidence | Empty/insufficient evidence bundle | Produce explicit insufficient-evidence answer instead of bluffing | Ask user for a narrower question or gather better evidence next turn |
| Event stream bloat | Oversized event payloads | Truncate low-value detail while preserving step labels and key outputs | Keep REPL readable without hiding the major steps |
