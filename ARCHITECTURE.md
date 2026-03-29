# Paddles Architecture: Recursive Harness Backbone

This document describes the implemented recursive harness backbone of `paddles` and the remaining experimental edges around it.

## Backbone Architecture

`paddles` should operate as a recursive in-context planning harness.

The backbone has five layers:

1. `InterpretationContextAssembler`
   Builds turn-time context from `AGENTS.md`, linked foundational docs, recent
   turns, retained evidence, and prior tool outputs.

2. `PlannerLane`
   A planner-capable model chooses the next bounded action before route
   selection: answer, tool, search, read, inspect, refine, branch, or stop.

3. `RecursiveExecutionLoop`
   Validates and executes planner actions, appends outputs back into context,
   and repeats until evidence is sufficient or budgets are exhausted.

4. `SynthesisLane`
   A separate answer model produces the final user-facing response from the
   accumulated planner trace and evidence bundle.

5. `Renderer`
   The TUI/plain output surfaces show the recursive work and final answer.

6. `RecorderBoundary`
   A paddles-owned trace contract projects the same runtime transitions into a
   `TraceRecorder` port, with transcript rendering staying a projection rather
   than the durable source of truth.

## Why This Shape

This architecture exists to solve three failures of one-shot small-model
interaction:

- the model answers before it has recursively gathered enough context
- operator memory influences style late, but not interpretation early
- retrieval and final synthesis are treated as the same workload

The recursive harness fixes that by putting recursive context work in front of
final synthesis.

## Core Rules

- Interpretation should happen before routing commits to a path.
- Top-level routing should come from a constrained model-selected next action, not a controller string heuristic.
- `AGENTS.md` is part of interpretation context, not just answer prompting.
- Planner and synthesizer are different roles and may use different models.
- Keel is an evidence domain, not a first-class runtime intent.
- Recursive planning must stay bounded and observable.
- Local-first remains the default operating mode.

## Planner Loop

The planner loop is the heart of the backbone:

1. assemble interpretation context
2. ask the planner for the next bounded action
3. validate the action
4. execute it
5. append outputs into loop state
6. repeat or stop
7. synthesize the final answer from the resulting evidence

The planner does not get to execute arbitrary unconstrained behavior. It stays
inside bounded action and budget contracts.

## Planner Actions

The planner boundary should be able to express actions like:

- answer or synthesize now
- bridge into deterministic tool execution when the controller/tool runtime is the best next step
- search the workspace
- read a file or artifact
- inspect prior tool output
- refine a search query
- branch an investigation into subqueries
- stop and request synthesis

Those actions can be backed by Sift search, workspace tools, retained artifacts,
or future planner-capable providers such as `context-1`.

## Planner And Synthesizer Separation

This split is important:

- the planner is optimized for recursive resource use
- the synthesizer is optimized for final answer quality

That means:

- the best planner model may not be the best answer model
- planner traces should not be treated as final user answers
- routing should be able to select different providers for each role

## Keel And Project Context

Keel is not special-cased as a first-class intent in the backbone design.

Mission files, charters, PRDs, voyage docs, and board commands should enter the
planner the same way other project evidence enters it:

- through memory
- through search
- through file reads
- through tool outputs
- through retained evidence from prior recursive steps

That preserves generality and keeps `paddles` useful outside one board engine.

## Current Implementation Snapshot

The repo now contains the main pieces of the target architecture:

- a controller-owned runtime in [src/application/mod.rs](/home/alex/workspace/spoke-sh/paddles/src/application/mod.rs)
- typed turn and planner events in [src/domain/model/turns.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/turns.rs)
- a planner contract in [src/domain/ports/planning.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/ports/planning.rs)
- a Sift-backed planner/synth model adapter in [src/infrastructure/adapters/sift_agent.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_agent.rs) and [src/infrastructure/adapters/sift_planner.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_planner.rs)
- a local gatherer backend in [src/infrastructure/adapters/sift_autonomous_gatherer.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_autonomous_gatherer.rs)
- interpretation-time operator memory in [src/infrastructure/adapters/agent_memory.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/agent_memory.rs)
- a default transcript TUI in [src/infrastructure/cli/interactive_tui.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/cli/interactive_tui.rs)
- a paddles-owned trace contract in [src/domain/model/traces.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/traces.rs)
- a recorder port in [src/domain/ports/trace_recording.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/ports/trace_recording.rs)
- embedded/noop/in-memory recorder adapters in [src/infrastructure/adapters/trace_recorders.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/trace_recorders.rs)

The remaining transitional pieces are now smaller:

- the main mech-suit service selects the first bounded action through the planner lane, but a temporary `tool` action still bridges into the older deterministic tool runtime
- legacy direct adapter helpers still carry heuristic intent inference outside the normal `MechSuitService` path
- planner `search` / `refine` actions currently delegate to the configured gatherer backend rather than a richer unified resource graph
- the default `sift-autonomous` gatherer path now runs bounded graph-mode retrieval for recursive planner `search` / `refine` work and preserves graph episode/frontier/branch state as typed `paddles` metadata
- the recorder boundary is live, but the default runtime still uses the noop recorder until a policy/config slice chooses an always-on local recorder
- artifact envelopes keep large payloads behind logical ids and optional locators, but there is no external artifact-store promotion policy yet
- `context-1` remains an explicit experimental boundary

## Recorder Boundary

The recorder path is now:

1. runtime transitions create typed `TraceRecord` values
2. transcript rendering still flows through `TurnEventSink`
3. durable recording flows through `TraceRecorder`
4. noop and in-memory adapters preserve local safety and tests
5. embedded `transit-core` maps roots, branch heads, appends, replay, and
   checkpoints without leaking raw transit types into the domain

This keeps the domain storage-neutral while making lineage durable enough for
later replay, branch comparison, and graph-trace analysis.

## Transitional Gap: Tool Bridge And Legacy Helpers

The main runtime path now follows the target backbone shape:

1. assemble interpretation context
2. ask the model for the first bounded action
3. validate and execute it
4. recurse until synthesis is appropriate

The remaining mismatch is narrower:

1. a selected `tool` action still hands off to the existing deterministic tool runtime instead of a unified planner resource graph
2. some legacy direct adapter helper methods still infer intent heuristically when called outside `MechSuitService`

That means the backbone contract is delivered for the primary interactive and
`process_prompt` runtime, while a few compatibility surfaces still need to be
folded into the same model-directed action system.

## Current Model Routing

Current routing now uses explicit planner/synth roles:

- synthesizer default: `qwen-1.5b`
- planner default: synthesizer model unless `--planner-model` overrides it
- optional coding-oriented planner/synth models: `qwen-coder-0.5b`,
  `qwen-coder-1.5b`, `qwen-coder-3b`
- heavier opt-in planner/synth lane: `qwen3.5-2b`
- local gatherer backend: `sift-autonomous`
- current recursive gatherer mode: bounded `graph` for planner-driven `search` / `refine` requests
- experimental planner/gatherer boundary: `context-1`

## Context-1 Fit

`context-1` belongs on the planner side of the architecture, not as the default
answer model.

It is a candidate specialized planner/gatherer lane because the recursive loop
is fundamentally about iterative retrieval, pruning, and refinement. The final
answer should still come from a separate synthesizer contract.

## Documentation Contract

Because the implementation still has experimental edges, the docs must stay honest:

- README explains the backbone architecture and current status at a high level
- ARCHITECTURE.md explains the target/current split in more detail
- POLICY.md captures the invariants that should govern the transition
- AGENTS.md keeps operator guidance aligned with those invariants

Do not document the recursive planner loop as fully delivered until mission
`VFDv1ha1G` is verified.
