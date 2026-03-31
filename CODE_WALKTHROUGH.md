# Paddles Code Walkthrough

This document orients contributors and agents to the source layout, key abstractions, and data flows in the paddles codebase. For governance philosophy see [CONSTITUTION.md](CONSTITUTION.md); for architectural contracts see [ARCHITECTURE.md](ARCHITECTURE.md).

## Workspace Layout

Paddles is a Rust workspace with two crates:

| Crate | Path | Purpose |
|-------|------|---------|
| **paddles** | `src/` | Binary entry point, domain model, application services, infrastructure adapters, CLI, and web UI |
| **paddles-conversation** | `crates/paddles-conversation/` | Shared conversation primitives: context tiers, locators, artifact envelopes, thread/session types |

`paddles` depends on `paddles-conversation`. The conversation crate has no paddles-specific infrastructure dependencies — it defines the data contract that flows between the harness and any recording or resolution backend.

## Three-Layer Architecture

```
CLI / Web  →  Application  →  Domain  ←  Infrastructure
```

| Layer | Location | Responsibility |
|-------|----------|----------------|
| **Domain** | `src/domain/` | Model types (turns, traces, threading, compaction, context quality), hexagonal ports (planning, gathering, synthesis, recording, resolution) |
| **Application** | `src/application/` | `MechSuitService` — session-scoped turn orchestration, interpretation assembly, planner/synthesizer dispatch, trace recording |
| **Infrastructure** | `src/infrastructure/` | Adapters (Sift planner, synthesizer, gatherer, HTTP providers, agent memory, trace recorders, transit resolver), CLI/TUI, web server, config, credentials |

### Domain Layer

```
domain/
├── model/
│   ├── turns.rs            # TurnEvent enum, TurnIntent, TurnEventSink trait
│   ├── traces.rs           # TraceRecord, TraceLineage, TraceBranch, artifact envelopes
│   ├── threading.rs        # ConversationReplayView, thread merge/split projection
│   ├── compaction.rs       # CompactionPlan, CompactionRequest, CompactionDecision
│   ├── context_quality.rs  # ContextPressure, PressureTracker, PressureLevel, PressureFactor
│   └── interpretation.rs   # InterpretationContext, guidance documents, tool hints, procedures
└── ports/
    ├── planning.rs          # PlannerAction schema, PlannerRequest, RecursiveExecutionLoop contract
    ├── context_gathering.rs # ContextGatherRequest, EvidenceBundle, retrieval strategy
    ├── context_resolution.rs# ContextResolver trait — cross-tier locator resolution
    ├── synthesis.rs         # Synthesis lane contract
    ├── operator_memory.rs   # OperatorMemory trait — AGENTS.md hierarchy
    └── trace_recording.rs   # TraceRecorder trait — noop, in-memory, transit-core
```

### Key Domain Types

| Type | Module | Role |
|------|--------|------|
| **TurnEvent** | `turns.rs` | Typed event enum for every observable step: classification, routing, planner actions, tool calls, context pressure, synthesis |
| **TurnIntent** | `turns.rs` | Classified intent: Casual, DirectResponse, DeterministicAction, Planned |
| **TraceRecord** | `traces.rs` | Immutable trace entry with stable task/turn/record/branch ids and typed lineage |
| **ArtifactEnvelope** | `paddles-conversation` | Inline content + optional typed `ContextLocator` pointing to full content in a deeper tier |
| **ContextTier** | `paddles-conversation` | Four-tier model: Inline, Transit, Sift, Filesystem |
| **ContextLocator** | `paddles-conversation` | Tagged union encoding which tier holds the full content and how to reach it |
| **ContextPressure** | `context_quality.rs` | Aggregated context degradation signal: pressure level + truncation count + contributing factors |
| **CompactionPlan** | `compaction.rs` | Structured plan for pruning low-value retained evidence while preserving locators |
| **InterpretationContext** | `interpretation.rs` | Assembled context: summary, guidance documents, tool hints, decision procedures |
| **PlannerAction** | `planning.rs` | Bounded action schema: answer, search, read, inspect, shell, refine, branch, stop |

### Application Layer

`src/application/mod.rs` contains `MechSuitService`, the session-scoped service that orchestrates each turn:

1. **Interpretation assembly** — loads AGENTS.md memory, derives guidance subgraph, assembles tool hints and procedures
2. **Intent classification** — model classifies the prompt as casual, direct, deterministic, or planned
3. **Routing** — dispatches to planner loop or direct synthesis based on intent and initial action selection
4. **Planner loop** — `RecursiveExecutionLoop` drives bounded action cycles through the planner port
5. **Synthesis** — separate synthesizer lane produces grounded answer from evidence bundle
6. **Recording** — trace records and turn events flow through `TraceRecorder` and `TurnEventSink`

### Infrastructure Adapters

| Adapter | File | Purpose |
|---------|------|---------|
| **SiftAgent** | `sift_agent.rs` | Primary synthesizer: conversation management, tool execution, context assembly, model interaction |
| **SiftPlanner** | `sift_planner.rs` | Planner model adapter: action selection, budget enforcement, graph-mode retrieval |
| **SiftAutonomousGatherer** | `sift_autonomous_gatherer.rs` | Local retrieval backend for planner search/refine actions |
| **AgentMemory** | `agent_memory.rs` | AGENTS.md hierarchy loader with truncation tracking and guidance document resolution |
| **HttpProvider** | `http_provider.rs` | Multi-provider HTTP adapter: OpenAI, Anthropic, Gemini API protocols |
| **TransitContextResolver** | `transit_resolver.rs` | Cross-tier locator resolution: inline, transit replay, filesystem read |
| **TraceRecorders** | `trace_recorders.rs` | Noop, in-memory, and embedded transit-core recording backends |
| **InteractiveTUI** | `interactive_tui.rs` | Ratatui-based TUI: transcript stream, spinner, in-flight indicators, timing, prompt composer |

## Context Architecture

Paddles manages context across four tiers:

```
Inline  ──locator──▸  Transit  ──file ref──▸  Filesystem
                         │
                    (Sift: future)
```

- **Inline** — character-limited excerpts in `ArtifactEnvelope.inline_content`
- **Transit** — full trace records replayed via `TransitContextResolver`
- **Filesystem** — workspace files read on demand
- **Sift** — indexed retrieval (future, currently returns explicit unsupported error)

When inline content is truncated, the envelope carries a typed `ContextLocator` pointing to the full record. Resolution is lazy — consumers call `resolver.resolve(&locator)` on demand. If a tier is unavailable, resolution fails closed with an explicit error.

A `PressureTracker` accumulates truncation events during context assembly and emits `ContextPressure` as a turn event when pressure is non-nominal.

## Typical Turn Flow

Example: user asks "What changed in the last commit?"

1. **Load** — `AgentMemory::load()` discovers AGENTS.md files from system, user, and ancestor scopes
2. **Interpret** — model-derived guidance subgraph expands tool hints and decision procedures from the memory roots
3. **Classify** — intent classified as `Planned` (workspace investigation needed)
4. **Route** — initial action model selects `search` as the first planner action
5. **Plan** — `RecursiveExecutionLoop` executes: search → read → answer
6. **Gather** — each search/refine action dispatches through `SiftAutonomousGatherer` in bounded graph mode
7. **Record** — each step produces a `TraceRecord` flowing through the recorder boundary
8. **Emit** — each step emits a `TurnEvent` flowing through the event sink to the TUI transcript
9. **Pressure** — `PressureTracker` records any truncation events from memory or artifact assembly
10. **Synthesize** — synthesizer lane produces grounded answer with source citations from the evidence bundle
11. **Render** — TUI reveals the answer with character-by-character animation

## TUI Event Stream

The interactive TUI in `interactive_tui.rs` renders turn events as a scrolling transcript:

- Events arrive through an `mpsc` channel from the async turn task
- Each event is formatted into a `TranscriptRow` with header, content, and optional timing
- Timing tracks elapsed time, step deltas, and pace classification (fast/normal/slow)
- Progress events (planner steps, search phases) replace their previous row in-place
- After 2s of silence during a busy turn, a contextual in-flight row appears ("Planning...", "Synthesizing...", etc.)
- Completed transcript rows flush to terminal scrollback; the live viewport stays compact

## Where to Look

| I want to... | Start here |
|--------------|-----------|
| Understand the turn loop | `src/application/mod.rs` → `MechSuitService` |
| Add a new planner action | `src/domain/ports/planning.rs` → `PlannerAction` enum |
| Change how events render in TUI | `src/infrastructure/cli/interactive_tui.rs` → `format_turn_event_row` |
| Add a new turn event type | `src/domain/model/turns.rs` → `TurnEvent` enum |
| Modify operator memory loading | `src/infrastructure/adapters/agent_memory.rs` |
| Change context resolution | `src/infrastructure/adapters/transit_resolver.rs` |
| Add a new model provider | `src/infrastructure/adapters/http_provider.rs` |
| Modify trace recording | `src/domain/ports/trace_recording.rs` + `src/infrastructure/adapters/trace_recorders.rs` |
| Change context pressure logic | `src/domain/model/context_quality.rs` |
| Modify compaction behavior | `src/domain/model/compaction.rs` |
| Add shared conversation types | `crates/paddles-conversation/src/lib.rs` |
