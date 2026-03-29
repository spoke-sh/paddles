# Paddles: Recursive In-Context Planning Harness

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Nix Build](https://img.shields.io/badge/Nix-Build-informational)](https://nixos.org/guides/how-nix-works)
[![Keel Board](https://img.shields.io/badge/Keel-Board-blue)](.keel/README.md)

> `paddles` is the mech suit around a local-first coding agent. Its backbone architecture is a recursive in-context planning harness: operator memory shapes turn interpretation, a planner model recursively gathers and refines evidence through bounded resource use, and a separate synthesizer model produces the final answer from that trace.

## Backbone Architecture

The central idea is simple:

- do not hardcode domain-specific turn types as the primary reasoning engine
- do not let controller heuristics commit the route before the model has seen interpretation context and chosen its next bounded action
- do not ask one small model to answer before it has done enough recursive context work
- do not treat retrieval and final answering as the same workload

Instead, `paddles` should behave like a bounded recursive harness.

### Recursive Loop

```mermaid
flowchart TD
    U["User Turn"]
    I["Interpretation Context<br/>AGENTS.md + linked docs + recent turns + prior tool state"]
    P["Planner Lane<br/>planner-capable model"]
    D{"Next bounded action?"}
    A["Validated resource action<br/>search / read / inspect / refine / branch"]
    X["Deterministic tool bridge<br/>legacy tool runtime"]
    S["Planner Trace + Evidence State"]
    T["Stop condition<br/>enough evidence / budget exhausted / explicit stop"]
    Y["Synthesizer Lane<br/>final answer model"]
    R["TUI / Plain Output"]

    U --> I --> P --> D
    D -->|answer| Y
    D -->|tool| X --> Y
    D -->|search/read/refine| A --> S --> I
    D -->|stop| T --> Y
    S --> T
    Y --> R
```

### Model Routing

Routing is workload-specific. The point is not to find one default model for everything. The point is to route each phase of the turn to the smallest capable lane.

```mermaid
flowchart LR
    Turn["Incoming Turn"]
    Interpret["Interpretation context<br/>AGENTS.md + linked docs + recent turns + local state"]
    Decide["Model-selected next action<br/>bounded schema"]
    Direct["Answer/synthesize now"]
    Tool["Deterministic tool bridge"]
    Resource["Validated resource action<br/>search / read / inspect / refine / branch"]
    LocalPlanner["Local planner path<br/>Sift autonomous / local Qwen planner"]
    HeavyPlanner["Specialized planner path<br/>Context-1 boundary or future planner model"]
    Evidence["Trace + Evidence Bundle"]
    Synth["Synthesizer lane<br/>grounded final answer"]

    Turn --> Interpret --> Decide
    Decide --> Direct --> Synth
    Decide --> Tool --> Synth
    Decide --> Resource
    Resource --> LocalPlanner --> Evidence
    Resource --> HeavyPlanner --> Evidence
    Evidence --> Decide
    Evidence --> Synth
```

### Delivered Backbone Step

The primary mech-suit path now assembles interpretation context first and asks
the planner model to choose the first bounded action before the controller
commits to a route. The controller still owns schema validation, allowlists,
budgets, and fail-closed behavior, but it no longer heuristically decides the
initial path for normal turns.

```mermaid
flowchart TD
    U["User turn"]
    I["Interpretation context<br/>AGENTS.md + linked docs + turns + local state"]
    M["Action-selection model"]
    C{"Choose one"}
    A["answer / synthesize"]
    T["tool bridge"]
    R["search / read / inspect / refine / branch"]
    L["validated recursive loop"]
    S["grounded synthesizer answer"]

    U --> I --> M --> C
    C --> A --> S
    C --> T --> S
    C --> R --> L --> M
    L --> S
```

### Role Of Keel

Keel is important, but it is not supposed to become a first-class special-case runtime intent. It is one evidence domain inside the workspace. Mission files, charters, PRDs, voyage docs, and board commands should be reachable through the same recursive context mechanisms as source files and tool outputs.

That is why the next architecture step is not “add a board intent.” It is “make the planner better at using recursive context.”

## Current Implementation Snapshot

The repository now implements the recursive harness in a bounded local-first form.

Today, the runtime has:

- a planner lane that sees interpretation context before choosing the first bounded action
- hierarchical `AGENTS.md` reload plus linked foundational guidance excerpts at interpretation time
- a model-directed first action schema that can choose `answer`, `tool`, `search`, `read`, `inspect`, `refine`, `branch`, or `stop`
- a bounded recursive loop for `search`, `read`, `inspect`, `refine`, `branch`, and `stop`
- a distinct synthesizer lane that answers from the resulting evidence bundle
- a default TUI/event stream that shows interpretation, planner actions, retrieval, fallbacks, and synthesis

The remaining gaps are narrower now:

- the `tool` initial action is still a transitional bridge into the older deterministic tool runtime instead of a fully unified planner resource graph
- legacy direct adapter helpers outside the main mech-suit service still contain heuristic intent inference and should not be treated as the backbone contract
- the recursive loop currently relies on the configured gatherer backend for workspace search rather than a richer unified resource graph
- graph-mode gatherer traces remain inline today; the domain contract leaves room for future external artifact references, but no embedded recorder is wired yet
- `context-1` is still an explicit experimental boundary, not the default planner lane

## Design Principles

- `AGENTS.md` should influence interpretation, not just answer style.
- The model should choose the next bounded action from interpretation context; the controller should validate and execute it safely.
- Recursive context refinement should do the heavy lifting for difficult workspace questions.
- Planner and synthesizer are different roles and may use different models.
- Keel and other project-specific artifacts are context, not hardcoded product logic.
- Local-first remains the default. Heavier planner lanes must degrade safely.
- Operator-visible traces matter. The harness should show its recursive work.

## Current Runtime Lanes

- The synthesizer lane defaults to `qwen-1.5b`.
- The planner lane defaults to the synthesizer model unless `--planner-model <id>` selects a different planner-capable model.
- `qwen-coder-0.5b`, `qwen-coder-1.5b`, `qwen-coder-3b`, and `qwen3.5-2b` remain available as opt-in planner or synthesizer variants.
- `sift-autonomous` is the current local gatherer/search backend used by planner `search` and `refine` actions.
- Recursive planner `search` and `refine` actions now request bounded `graph` mode through that gatherer path instead of stopping at linear autonomous search.
- Graph-mode gatherer results preserve typed branch/frontier/node/edge metadata with stable ids in the evidence bundle and default event stream.
- `context-1` remains an explicit experimental planner/gatherer boundary and stays fail-closed until its harness is real.

## Foundational Documents

Use these in this order when interpreting the mech suit:

- [AGENTS.md](AGENTS.md) for operator guidance and the canonical turn loop
- [README.md](README.md) for the backbone architecture and document map
- [ARCHITECTURE.md](ARCHITECTURE.md) for the detailed target/current architecture split
- [POLICY.md](POLICY.md) for runtime invariants and safety rules
- [INSTRUCTIONS.md](INSTRUCTIONS.md) for procedural Keel loops
- [CONFIGURATION.md](CONFIGURATION.md) for lane/runtime configuration
- [PROTOCOL.md](PROTOCOL.md) for communications and data contracts
- [CONSTITUTION.md](CONSTITUTION.md) for collaboration philosophy and decision hierarchy

## Working With The Board

Use the raw `keel` CLI directly.

The normal operator rhythm is:

1. Orient with `keel health --scene`, `keel flow --scene`, and `keel doctor --status`.
2. Inspect with `keel mission next --status`, `keel pulse`, and `keel workshop`.
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
keel doctor --status
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

Later files are more specific. That memory now participates in turn interpretation before planner action selection, and linked foundational docs are pulled in as compact excerpts rather than late prompt-only baggage.

## Why This Architecture

The goal is to raise the effective performance of smaller local models through recursive resource use rather than by hardcoding project-specific turn classes or jumping immediately to a larger answer model.

That is the mech suit:

- human-authored guidance and architecture
- bounded recursive planning
- explicit evidence accumulation
- separate final synthesis
- visible execution

## License

MIT. See [LICENSE](LICENSE).
