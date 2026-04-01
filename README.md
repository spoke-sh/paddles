# Paddles: Recursive In-Context Planning Harness

[![Keel Board](https://img.shields.io/badge/Keel-Board-blue)](.keel/README.md)
[![CI](https://github.com/spoke-sh/paddles/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/spoke-sh/paddles/actions/workflows/ci.yml)

> `paddles` is the mech suit around a local-first coding agent. Its backbone architecture is a recursive in-context planning harness: operator memory shapes turn interpretation, a planner model recursively gathers and refines evidence through bounded resource use, and a separate synthesizer model produces the final answer from that trace.

## Backbone Architecture

The architecture rests on four commitments:

- **Let the model reason first.** Interpretation context — operator memory, guidance graphs, tool hints, derived procedures — arrives before any routing decision. The model sees the full picture and chooses its own next action.
- **Earn the answer through recursive work.** Small models become dramatically more capable when the harness gives them bounded tools to gather evidence iteratively rather than answering in one shot.
- **Separate planning from synthesis.** Recursive context gathering and final answer generation are distinct workloads, each routed to the smallest model that excels at that role.
- **Keep every step visible.** The harness shows its recursive work — planner actions, evidence gathered, decisions made — so the operator always knows why an answer was produced.

### The Turn Loop

Every interaction follows the same recursive cycle. The harness assembles context, the planner model decides what to do next, the controller validates and executes that action, and the loop continues until evidence is sufficient for synthesis.

```mermaid
flowchart TD
    U["🎯 User Turn"]
    I["📖 Interpretation Context<br/>operator memory · guidance graph · tool hints<br/>derived procedures · recent turns · prior evidence"]
    P["🧠 Planner Lane<br/>model chooses next bounded action"]
    D{"What does the<br/>evidence need?"}
    W["🔧 Workspace Action<br/>search · read · inspect · shell · diff · edit"]
    R["🔄 Refine / Branch<br/>sharpen the query · split into subqueries"]
    E["📋 Evidence Accumulates<br/>planner trace · tool outputs · typed metadata"]
    T["✅ Synthesis Ready<br/>enough evidence · budget met · explicit stop"]
    S["💬 Synthesizer Lane<br/>grounded final answer from evidence bundle"]
    O["📺 Visible Output<br/>TUI transcript or plain CLI"]

    U --> I --> P --> D
    D -->|"answer directly"| T
    D -->|"gather more"| W --> E
    D -->|"refine focus"| R --> E
    E -->|"loop back"| I
    E -->|"sufficient"| T
    T --> S --> O

    style U fill:#e8f4f8,stroke:#2196F3
    style I fill:#fff3e0,stroke:#FF9800
    style P fill:#f3e5f5,stroke:#9C27B0
    style S fill:#e8f5e9,stroke:#4CAF50
    style O fill:#e8f4f8,stroke:#2196F3
```

This is the heart of Paddles: a bounded recursive loop where the model drives its own investigation within safe guardrails. Each pass through the loop adds evidence, refines understanding, and brings the answer closer to ground truth.

### Model Routing

Each phase of the turn flows to the smallest model capable of that workload. A lightweight synthesizer handles final answers while a more capable planner drives the recursive investigation.

```mermaid
flowchart LR
    Turn["Incoming Turn"]

    subgraph Interpret ["Interpretation"]
        Memory["Operator memory<br/>AGENTS.md hierarchy"]
        Graph["Guidance graph<br/>model-derived subgraph"]
        Hints["Tool hints +<br/>derived procedures"]
    end

    subgraph Plan ["Recursive Planning"]
        Decide["Model selects<br/>next bounded action"]
        Local["Local planner<br/>Sift / Qwen"]
        Heavy["Specialized planner<br/>Context-1 boundary"]
        Evidence["Evidence bundle<br/>typed trace + metadata"]
    end

    subgraph Finish ["Synthesis"]
        Synth["Synthesizer lane<br/>grounded cited answer"]
    end

    Turn --> Interpret --> Decide
    Decide -->|"workspace action"| Local --> Evidence
    Decide -->|"specialized retrieval"| Heavy --> Evidence
    Evidence -->|"loop back"| Decide
    Evidence -->|"ready"| Synth
    Decide -->|"answer directly"| Synth
```

### The Recursive Harness in Practice

The primary mech-suit path assembles interpretation context first, then lets the planner model choose each bounded action. The controller validates, executes, and enforces budgets — the model drives direction, the controller ensures safety.

```mermaid
flowchart TD
    U["User turn arrives"]
    I["Assemble interpretation context<br/>operator memory · guidance graph · tool hints · procedures"]
    M["Planner model evaluates context"]
    C{"Model's choice"}
    A["Answer — evidence is sufficient"]
    T["Act — workspace action needed"]
    R["Recurse — search · refine · branch"]
    L["Execute within validated loop<br/>controller enforces schema + budgets"]
    S["Synthesizer produces grounded answer<br/>with source citations"]

    U --> I --> M --> C
    C --> A --> S
    C --> T --> S
    C --> R --> L --> M
    L --> S

    style M fill:#f3e5f5,stroke:#9C27B0
    style L fill:#fff3e0,stroke:#FF9800
    style S fill:#e8f5e9,stroke:#4CAF50
```

### Trace Recording

Every recursive step produces typed trace records alongside the visible transcript. The UI is a projection; durable lineage lives in the recorder boundary.

```mermaid
flowchart LR
    Runtime["Runtime transitions<br/>planner · gatherer · tool · synth"]

    subgraph Recording ["Durable Trace Path"]
        Contract["Trace contract<br/>task root · action · branch · checkpoint"]
        Port["TraceRecorder port"]
        Adapters["Noop · In-memory · transit-core"]
    end

    subgraph Display ["Operator View"]
        Events["Turn events → TUI transcript"]
    end

    Runtime --> Contract --> Port --> Adapters
    Runtime --> Events
    Contract -.->|"projects to"| Events
```

### Threaded Conversations

Interactive sessions maintain one durable conversation root. When a steering prompt arrives mid-turn, the planner classifies it as continuation, child-thread, or merge-back — preserving full lineage for replay.

```mermaid
flowchart TD
    U["Active turn running"]
    Q["Steering prompt captured<br/>structured thread candidate"]
    M["Thread decision model<br/>evaluates context + thread state"]
    D{"Classification"}
    C["Continue — same thread"]
    B["Branch — open child thread<br/>new transit branch with lineage"]
    G["Merge — reconcile back<br/>explicit recorded outcome"]
    X["Execute on selected thread"]
    R["Thread-aware transcript<br/>full replay + branch history"]

    U --> Q --> M --> D
    D --> C --> X --> R
    D --> B --> X --> R
    D --> G --> X --> R

    style M fill:#f3e5f5,stroke:#9C27B0
    style R fill:#e8f5e9,stroke:#4CAF50
```

## What The Harness Delivers Today

The recursive harness runs as a bounded local-first runtime:

- **Interpretation-first routing** — every turn assembles operator memory, a model-derived guidance subgraph, read-only tool hints, and derived decision procedures before the planner chooses its first action
- **Model-driven action selection** — the planner chooses from `answer`, workspace actions (`search`, `list_files`, `read`, `inspect`, `shell`, `diff`, `write_file`, `replace_in_file`, `apply_patch`), `refine`, `branch`, or `stop`
- **Guidance-aware fallbacks** — fallback selection draws on command hints and decision procedures from foundational docs, and halts recursion when a procedure step has already resolved the request
- **Bounded recursive loop** — workspace actions, refinements, and branches all feed back into the planner until evidence is sufficient or budgets are met
- **Separate synthesis** — a distinct synthesizer lane produces the final grounded answer from the accumulated evidence bundle
- **Full-stream visibility** — a default TUI/event stream shows interpretation, planner actions, retrieval, fallbacks, and synthesis as they happen
- **Durable trace lineage** — a paddles-owned trace contract with stable task/turn/record/branch/checkpoint ids, backed by a `TraceRecorder` boundary with noop, in-memory, and embedded `transit-core` adapters
- **Artifact envelopes** — prompts, tool I/O, evidence bundles, planner traces, and responses sit behind logical refs, ready for external storage when needed
- **Threaded conversations** — interactive sessions keep one durable root task with model-driven steering-thread decisions, structured thread candidates, explicit merge-back records, and full replay views
- **Four-tier context model** — context spans Inline, Transit, Sift, and Filesystem tiers with typed `ContextLocator` addressing and lazy cross-tier resolution through a `ContextResolver` port
- **Transit-native addressing** — truncated `ArtifactEnvelope` content carries typed locators to full records in transit or on disk, resolvable on demand without re-searching
- **Recursive self-assessing compaction** — the planner evaluates its own retained evidence for relevance and produces structured compaction plans that prune low-value artifacts while preserving locators for depth
- **Context pressure signals** — a `PressureTracker` accumulates truncation events during context assembly and emits `ContextPressure` turn events so context degradation is visible in the event stream
- **In-flight visibility** — the TUI inserts contextual "Planning...", "Synthesizing...", etc. rows after 2s of silence between events, so long model calls don't look stalled
- **Shared conversation primitives** — an internal workspace crate ([crates/paddles-conversation/src/lib.rs](/home/alex/workspace/spoke-sh/paddles/crates/paddles-conversation/src/lib.rs)) cleanly separates conversation/thread/session types from the main binary

### Growing Edges

A few areas are still maturing:

- **Sift-tier locator resolution** — typed `ContextLocator::Sift` values are emitted from retrieval; direct Sift resolver wiring is still being finalized
- **Automatic tier promotion** — content moves between tiers through explicit locators; automatic promotion/demotion policies are future work
- **Default recording policy** — embedded `transit-core` recording is available through the recorder boundary; the default runtime still uses noop until the policy slice lands
- **Context-1 integration** — `context-1` remains an explicit experimental boundary, available for opt-in use
- **Concurrent threading** — auto-threading is checkpoint-bounded and sequential today; true concurrent sibling generation is a future capability

## Design Principles

- **Interpretation shapes direction.** `AGENTS.md` memory influences what the planner investigates, how it prioritizes, and which procedures it follows.
- **The model drives, the controller guards.** The model selects its next bounded action from interpretation context; the controller validates, executes, and enforces budgets.
- **Recursive work earns better answers.** Difficult workspace questions improve through iterative evidence gathering rather than one-shot generation.
- **Separation of concerns.** Planner and synthesizer are distinct roles, potentially using different models optimized for their respective workloads.
- **Context over hardcoding.** Keel, project artifacts, and board state flow through memory, search, and tool outputs — the harness stays general-purpose.
- **Local-first by default.** The core loop runs on local models. Heavier planner lanes are opt-in and degrade gracefully.
- **Visible execution.** Every recursive step is surfaced to the operator. The harness shows its work because transparency builds trust.

## Current Runtime Lanes

- The synthesizer lane defaults to `qwen-1.5b`.
- The planner lane defaults to the synthesizer model unless `--planner-model <id>` selects a different planner-capable model.
- `qwen-coder-0.5b`, `qwen-coder-1.5b`, `qwen-coder-3b`, and `qwen3.5-2b` remain available as opt-in planner or synthesizer variants.
- `sift-direct` is the default local gatherer/search backend used by planner `search` and `refine` actions.
- `paddles` owns recursive planning. `sift` executes direct retrieval only.
- Planner `search` and `refine` actions carry bounded retrieval mode and strategy into the gatherer boundary.
- Gatherer progress now reflects direct retrieval stages such as initialization, indexing, retrieval, and ranking.
- `context-1` remains an explicit experimental planner/gatherer boundary and stays fail-closed until its harness is real.

## Search And Retrieval

Search behavior is documented in [SEARCH.md](SEARCH.md).

Use that document when you need the retrieval boundary, provider names, capabilities, or constraints. The short version is:

- `paddles` plans
- `sift` retrieves
- `sift-direct` is the active local retrieval backend
- `sift-autonomous` remains a compatibility alias only

## Foundational Documents

Use these in this order when reading the foundational stack:

1. [AGENTS.md](AGENTS.md) for operator guidance and the top-level working contract
2. [INSTRUCTIONS.md](INSTRUCTIONS.md) for the canonical Keel turn loop and checklists
3. [README.md](README.md) for the backbone architecture and navigation map
4. [CONSTITUTION.md](CONSTITUTION.md) for collaboration philosophy and bounds
5. [POLICY.md](POLICY.md) for operational commitments and runtime guarantees
6. [ARCHITECTURE.md](ARCHITECTURE.md) for the turn loop narrative and implementation map
7. [PROTOCOL.md](PROTOCOL.md) for communications and data contracts
8. [SEARCH.md](SEARCH.md) for search/retrieval behavior, constraints, and provider semantics
9. [CONFIGURATION.md](CONFIGURATION.md) for concrete lane/runtime configuration

Supplementary references:

- [STAGE.md](STAGE.md) for visual philosophy
- [RELEASE.md](RELEASE.md) for release process
- [.keel/adrs/](.keel/adrs/) for binding architecture decisions

This reading order is not the same thing as the decision hierarchy. For ambiguous design decisions, defer to ADRs first, then Constitution, Policy, Architecture, and current planning artifacts.

## Working With The Board

Use the raw `keel` CLI directly.

The normal operator rhythm is:

1. Orient with `keel health --scene`, `keel flow --scene`, and `keel doctor`.
2. Inspect with `keel mission next`, `keel pulse`, and `keel workshop`.
3. Pull one slice with `keel next --role <role>` or by following the active mission/story explicitly.
4. Ship the slice and land a sealing commit.
5. Re-orient immediately after the commit.

## Development Setup

Enter the dev shell:

```bash
nix develop
```

Build and test:

```bash
just build
just test
just quality
```

Check board health:

```bash
keel doctor
keel flow --scene
```

Run the interactive assistant:

```bash
just paddles --cuda
```

Use a heavier planner lane while keeping a lighter synthesizer:

```bash
paddles --model qwen-1.5b --planner-model qwen3.5-2b
```

One-shot prompt mode stays plain for scripts:

```bash
paddles --prompt "Summarize the current runtime lanes"
```

## REPL Memory

`paddles` reloads `AGENTS.md` memory on every turn from:

1. `/etc/paddles/AGENTS.md`
2. `~/.config/paddles/AGENTS.md`
3. every ancestor `AGENTS.md` from filesystem root to the current workspace

Later files are more specific. That memory now participates in turn interpretation before planner action selection, and additional guidance is loaded through a turn-time model-derived subgraph rooted at `AGENTS.md` rather than a hardcoded foundational file list.

## Why This Architecture

The mech suit raises the effective performance of smaller local models through recursive resource use. A small model with bounded tools and iterative evidence gathering consistently outperforms the same model answering in one shot.

That is the mech suit:

- **Human-authored guidance** shapes every turn through operator memory and derived procedures
- **Bounded recursive planning** lets the model investigate iteratively within safe guardrails
- **Explicit evidence accumulation** grounds answers in real workspace artifacts
- **Separate final synthesis** optimizes each phase independently
- **Visible execution** makes every decision transparent and auditable

## License

MIT. See [LICENSE](LICENSE).
