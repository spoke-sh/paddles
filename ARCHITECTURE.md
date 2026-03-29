# Paddles Architecture: Recursive Harness Backbone

This document describes the intended backbone architecture of `paddles` and the current implementation snapshot while the recursive planner mission is still in flight.

## Backbone Architecture

`paddles` should operate as a recursive in-context planning harness.

The backbone has five layers:

1. `InterpretationContextAssembler`
   Builds turn-time context from `AGENTS.md`, linked foundational docs, recent
   turns, retained evidence, and prior tool outputs.

2. `PlannerLane`
   A planner-capable model chooses the next bounded action for non-trivial
   turns: search, read, inspect, refine, branch, or stop.

3. `RecursiveExecutionLoop`
   Validates and executes planner actions, appends outputs back into context,
   and repeats until evidence is sufficient or budgets are exhausted.

4. `SynthesisLane`
   A separate answer model produces the final user-facing response from the
   accumulated planner trace and evidence bundle.

5. `Renderer`
   The TUI/plain output surfaces show the recursive work and final answer.

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

The repo already contains several pieces of the target architecture:

- a controller-owned runtime in [src/application/mod.rs](/home/alex/workspace/spoke-sh/paddles/src/application/mod.rs)
- typed turn events in [src/domain/model/turns.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/turns.rs)
- a Sift-backed synthesizer/tool adapter in [src/infrastructure/adapters/sift_agent.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_agent.rs)
- a local autonomous gatherer in [src/infrastructure/adapters/sift_autonomous_gatherer.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_autonomous_gatherer.rs)
- hierarchical operator memory in [src/infrastructure/adapters/agent_memory.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/agent_memory.rs)
- a default transcript TUI in [src/infrastructure/cli/interactive_tui.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/cli/interactive_tui.rs)

But the current runtime is still transitional:

- static `TurnIntent` classification still exists and is controller-first
- `AGENTS.md` is loaded every turn, but mainly into prompt construction rather
  than first-pass interpretation
- repository questions usually get one gather pass rather than a true recursive
  planner loop
- planner and synthesizer roles are not yet cleanly separated

Mission [VFDv1ha1G](.keel/missions/VFDv1ha1G/README.md) exists to close that gap.

## Current Model Routing

Current routing still favors the existing synthesizer/gatherer split:

- synthesizer default: `qwen-1.5b`
- optional coding-oriented synthesizers: `qwen-coder-0.5b`,
  `qwen-coder-1.5b`, `qwen-coder-3b`
- heavier opt-in lane: `qwen3.5-2b`
- local gatherer provider: `sift-autonomous`
- experimental planner/gatherer boundary: `context-1`

The backbone direction is to evolve those lanes into explicit planner and
synthesizer roles instead of tying them to the older static controller shape.

## Context-1 Fit

`context-1` belongs on the planner side of the architecture, not as the default
answer model.

It is a candidate specialized planner/gatherer lane because the recursive loop
is fundamentally about iterative retrieval, pruning, and refinement. The final
answer should still come from a separate synthesizer contract.

## Documentation Contract

Because the implementation is mid-transition, the docs must stay honest:

- README explains the backbone architecture and current status at a high level
- ARCHITECTURE.md explains the target/current split in more detail
- POLICY.md captures the invariants that should govern the transition
- AGENTS.md keeps operator guidance aligned with those invariants

Do not document the recursive planner loop as fully delivered until mission
`VFDv1ha1G` is verified.
