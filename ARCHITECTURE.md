# Paddles Architecture: Recursive Harness Backbone

How `paddles` turns a user prompt into a grounded, evidence-backed answer through recursive in-context planning.

> Foundational stack position: `6/8`
> Read this after [POLICY.md](POLICY.md) and before [PROTOCOL.md](PROTOCOL.md).

## The Story of a Turn

Every turn through Paddles follows the same narrative arc: understand, investigate, and synthesize. The architecture exists to give small local models the structured support they need to produce answers that rival much larger models.

### Act 1: Interpretation

**`InterpretationContextAssembler`** builds the full picture before any decision is made.

The harness loads `AGENTS.md` operator memory from system, user, and ancestor scopes. A model-derived guidance subgraph discovers relevant procedures, tool hints, and project knowledge rooted at that memory. Recent turns, retained evidence, and prior tool state round out the context.

By the time the planner sees the prompt, it already knows the operator's priorities, the project's conventions, and the tools at its disposal.

### Act 2: Recursive Planning

**`PlannerLane`** drives an iterative investigation. The planner model evaluates the assembled context and selects its next bounded action: answer directly, search the workspace, read a file, inspect state, run a shell command, refine a query, branch into subqueries, or stop.

**`RecursiveExecutionLoop`** validates each action against schema and budget contracts, executes it safely, appends the output back into context, and loops. Each pass through the loop adds real evidence — file contents, search results, tool outputs — grounding the eventual answer in workspace reality.

The loop continues until the planner determines it has enough evidence, the budget is met, or an explicit stop is reached.

### Act 3: Synthesis

**`SynthesisLane`** takes the accumulated planner trace and evidence bundle and produces the final user-facing response. This is a separate model call optimized for answer quality, grounded in the concrete evidence the planner gathered. At boot, Paddles resolves a provider/model-specific render capability and then uses the strictest supported transport for final answers — native JSON schema or tool-call structure when available, prompt-enveloped JSON when not. Final answers still normalize through the same constrained render-envelope contract (`paragraph`, `bullet_list`, `code_block`, `citations`) before the UI projects them into transcript output.

### Visibility Throughout

**`Renderer`** surfaces every step of this process — interpretation assembly, planner action selection, gatherer work, tool calls, fallback decisions, and synthesis — through a TUI transcript or plain CLI output. The renderer consumes normalized assistant blocks rather than relying on ad hoc markdown conventions from the model. The interactive TUI uses a compact inline viewport with a borderless live tail above the boxed composer, so completed transcript rows stay in normal terminal scrollback instead of disappearing behind a single full-screen page.

**`RecorderBoundary`** captures the same runtime transitions as typed trace records with stable ids, flowing through a `TraceRecorder` port to noop, in-memory, or embedded `transit-core` adapters. The transcript UI is a projection of these records; durable lineage lives in the recorder.

**`ConversationThreadLayer`** maintains one durable conversation root across interactive sessions. Steering prompts become structured thread candidates, classified by a model into continuation, child-thread, or merge-back decisions — preserving full lineage for replay and analysis.

## Why This Shape Works

Three properties of this architecture compound to raise effective model performance:

1. **Interpretation arrives first.** Operator memory and project guidance shape the planner's priorities before it commits to any action. The model reasons within the operator's context from the start.
2. **Recursive evidence gathering earns the answer.** Instead of generating an answer from memory alone, the planner iteratively reads, searches, and refines until it has concrete evidence. Small models with tools consistently outperform the same models without them.
3. **Planning and synthesis are separate workloads.** The best recursive investigator may differ from the best answer composer. Separating these roles lets each be optimized independently and routed to the smallest capable model.

## Core Commitments

- **Interpretation before routing.** The model sees full context and chooses its own path.
- **Model-directed action selection.** The planner selects from a constrained action schema; the controller validates and executes.
- **`AGENTS.md` as the interpretation root.** Operator memory shapes investigation, priorities, and procedures — additional guidance flows through the model-derived graph.
- **Planner and synthesizer as distinct roles.** Each can use different models optimized for their workload.
- **Project artifacts as context.** Keel, board state, and domain knowledge enter through memory, search, and tools — keeping the harness general-purpose.
- **Bounded and observable recursion.** Every planner action is validated, budgeted, and visible to the operator.
- **Local-first by default.** The core loop runs on local models; heavier lanes are opt-in and degrade gracefully.

## The Planner Loop In Detail

The planner loop is the heart of the backbone. Each cycle follows a clear rhythm:

```mermaid
flowchart TD
    A["1. Assemble interpretation context"]
    B["2. Planner selects next bounded action"]
    C["3. Controller validates against schema + budget"]
    D["4. Execute the action safely"]
    E["5. Append outputs into loop state"]
    F{"6. Sufficient evidence?"}
    G["7. Synthesize grounded answer"]

    A --> B --> C --> D --> E --> F
    F -->|"continue"| A
    F -->|"ready"| G

    style A fill:#fff3e0,stroke:#FF9800
    style B fill:#f3e5f5,stroke:#9C27B0
    style C fill:#ffebee,stroke:#f44336
    style G fill:#e8f5e9,stroke:#4CAF50
```

The planner operates within bounded action and budget contracts. Every action is validated before execution, every output is recorded, and the operator can observe the full trace.

## Planner Action Vocabulary

The planner expresses its intentions through a constrained action schema:

| Action | Purpose |
|--------|---------|
| **answer** | Synthesize now — evidence is sufficient |
| **search** | Find relevant files or content in the workspace |
| **list_files** | Discover candidate files by pattern |
| **read** | Read a specific file or artifact |
| **inspect** | Examine read-only workspace state |
| **shell** / **diff** / **edit** | Execute concrete workspace modifications |
| **refine** | Sharpen a search query based on prior results |
| **branch** | Split an investigation into parallel subqueries |
| **stop** | Request synthesis with current evidence |

These actions are backed by Sift search, workspace tools, retained artifacts, and future planner-capable providers like `context-1`.

## The Value of Planner/Synthesizer Separation

Separating these roles unlocks three benefits:

- **Independent optimization** — the best recursive investigator and the best answer composer are often different models, and each can be routed to its ideal lane
- **Cleaner evidence flow** — planner traces are working artifacts that inform synthesis; the synthesizer transforms them into polished, grounded responses
- **Flexible routing** — operators can mix a lightweight synthesizer with a heavier planner, or vice versa, matching each role to available hardware

## Project Context as Evidence

Keel, board state, mission files, PRDs, and domain knowledge all enter the planner through the same channels as any other evidence:

- operator memory (AGENTS.md hierarchy)
- workspace search results
- file reads and tool outputs
- retained evidence from prior recursive steps

This keeps `paddles` general-purpose — useful with any project tooling, board engine, or domain.

## Implementation Map

The target architecture is implemented across these modules:

| Layer | Module | Role |
|-------|--------|------|
| **Runtime** | [src/application/mod.rs](/home/alex/workspace/spoke-sh/paddles/src/application/mod.rs) | Controller-owned service, session-scoped thread orchestration |
| **Turn Events** | [src/domain/model/turns.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/turns.rs) | Typed turn and planner event definitions |
| **Planner Contract** | [src/domain/ports/planning.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/ports/planning.rs) | Bounded action schema and planner port |
| **Planner Adapter** | [src/infrastructure/adapters/sift_planner.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_planner.rs) | Sift-backed planner model |
| **Synthesizer Adapter** | [src/infrastructure/adapters/sift_agent.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_agent.rs) | Sift-backed synthesis + guidance graph derivation |
| **Gatherer** | [src/infrastructure/adapters/sift_autonomous_gatherer.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/sift_autonomous_gatherer.rs) | Local graph-mode retrieval backend |
| **Operator Memory** | [src/infrastructure/adapters/agent_memory.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/agent_memory.rs) | AGENTS.md hierarchy + tool hint extraction + procedure derivation |
| **Trace Contract** | [src/domain/model/traces.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/traces.rs) | Stable task/turn/record/branch/checkpoint ids |
| **Recorder Port** | [src/domain/ports/trace_recording.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/ports/trace_recording.rs) | TraceRecorder boundary |
| **Recorder Adapters** | [src/infrastructure/adapters/trace_recorders.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/adapters/trace_recorders.rs) | Noop, in-memory, embedded transit-core |
| **Thread Replay** | [src/domain/model/threading.rs](/home/alex/workspace/spoke-sh/paddles/src/domain/model/threading.rs) | Replay/projection layer for conversation traces |
| **Conversation Crate** | [crates/paddles-conversation/src/lib.rs](/home/alex/workspace/spoke-sh/paddles/crates/paddles-conversation/src/lib.rs) | Shared conversation/thread/session primitives |
| **Transcript TUI** | [src/infrastructure/cli/interactive_tui.rs](/home/alex/workspace/spoke-sh/paddles/src/infrastructure/cli/interactive_tui.rs) | Default Codex-style action stream |

### How The Pieces Fit Together

The runtime follows the backbone narrative from above:

1. **Interpretation** — operator memory loads from the AGENTS.md hierarchy, then a model-derived guidance graph discovers tool hints and decision procedures. Invalid initial replies get one constrained re-decision pass before the controller fails closed.
2. **Planning** — workspace actions stay inside the planner loop. Search/refine actions carry model-selected retrieval mode and strategy into the gatherer boundary. The `sift-autonomous` gatherer runs bounded graph-mode retrieval, preserving episode/frontier/branch state as typed metadata.
3. **Recording** — the recorder boundary is live. Artifact envelopes keep large payloads behind logical ids with optional locators.
4. **Threading** — session-scoped orchestration uses the shared conversation crate for structured candidates, model-driven decisions, and explicit merge-back records.

### Growing Edges

- **Unified resource graph** — planner search/refine delegates through the configured gatherer; a richer resource graph will strengthen this
- **Default recorder policy** — embedded transit-core is available; the default runtime uses noop until the policy slice lands
- **Artifact store promotion** — the contract supports external artifact refs; a promotion policy will formalize this
- **Context-1 integration** — available as an explicit experimental boundary for opt-in use

## Recorder Boundary

The recorder path delivers storage-neutral trace durability:

1. Runtime transitions produce typed `TraceRecord` values
2. Transcript rendering flows through `TurnEventSink` as operator-facing projection
3. Durable recording flows through `TraceRecorder` as the source of truth
4. Noop and in-memory adapters preserve local safety and enable testing
5. Embedded `transit-core` maps roots, branch heads, appends, replay, and checkpoints — all through the domain boundary, keeping transit types internal
6. Interactive thread split/merge records flow through the same recorder path, making thread structure part of the durable trace

This design keeps the domain storage-neutral while providing lineage durable enough for replay, branch comparison, and graph-trace analysis.

## Current Backbone Status

The backbone contract is delivered for the primary interactive and `process_prompt` runtime:

1. Interpretation context assembles before routing
2. The model selects its first bounded action
3. The controller validates and executes it
4. The loop recurses until synthesis is appropriate

Two capabilities are still maturing:

- **Resource graph breadth** — planner search/refine delegates through the configured gatherer backend today; a unified resource graph will broaden what the planner can reach
- **Concurrent threading** — auto-threading replays and merges explicit thread lineage at safe checkpoints; true simultaneous sibling generation on one local model session is a future capability

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

`context-1` belongs on the planner side of the architecture — a candidate specialized planner/gatherer lane. The recursive loop is fundamentally about iterative retrieval, pruning, and refinement, which aligns with context-1's strengths. Final answers continue to come from the separate synthesizer contract.

## Documentation Contract

The foundational documents work together to tell the full story:

- **README** — backbone architecture narrative and current capabilities
- **ARCHITECTURE** — detailed turn loop, implementation map, and growing edges
- **POLICY** — operational commitments that govern the runtime
- **AGENTS** — operator guidance aligned with those commitments

The recursive planner loop tracks toward full delivery under mission `VFDv1ha1G`.
