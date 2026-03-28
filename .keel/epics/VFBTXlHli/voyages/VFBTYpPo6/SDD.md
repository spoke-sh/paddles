# Subagent Interface and Routing Foundations - Software Design Description

> Define a proper context-gathering subagent interface, evidence contract, and routing boundary so Paddles can later use Chroma Context-1 without replacing the default local answer runtime.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage introduces a controller-level split between:

- a default synthesizer lane for direct answers and existing tool-oriented turns
- a specialized context-gathering lane that can search, rank, and compress
  evidence before synthesis

The first implementation goal is architectural honesty, not full Chroma parity.
Paddles will define the subagent contract, route retrieval-heavy work to it,
and keep Context-1 behind an explicit experimental boundary until its harness
expectations can be met faithfully.

## Context & Boundaries

This design covers routing, gatherer contracts, evidence packaging, model lane
selection, and the Context-1 adapter boundary in `application`,
`infrastructure`, and foundational docs.

It does not cover replacing the default answer runtime, reimplementing Chroma's
private harness internals, or introducing mandatory remote execution for common
prompt handling.

```
┌──────────────────────────────────────────────────────────────┐
│                  Context-Gathering Lane Split               │
│                                                              │
│  User Turn                                                   │
│     │                                                        │
│     v                                                        │
│  Intent Router ───────────────┐                              │
│     │                          │                             │
│     │ direct/tool             │ retrieval-heavy             │
│     v                          v                             │
│  Synthesizer Lane       Context Gatherer Lane                │
│     │                          │                             │
│     │                          v                             │
│     └──────────────> Evidence Bundle ───────────────┐        │
│                                                      v       │
│                                              Final Synthesis │
└──────────────────────────────────────────────────────────────┘
             ↑                                   ↑
        Local workspace                    Experimental
        files and tools                    Context-1 adapter
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing local answer runtime | Internal runtime | Preserves the current direct-answer and tool-capable path | Current Paddles service |
| Context gatherer interface | Internal contract | Encapsulates retrieval work and evidence delivery | New voyage-owned boundary |
| Chroma Context-1 | Experimental model provider | Specialized context gathering for retrieval-heavy prompts | Hugging Face model card / Chroma research post |
| Local workspace search/filesystem tools | Host substrate | Provide a local gatherer implementation and evidence sources | Host-provided |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Lane split | Separate gatherer and synthesizer roles | Retrieval and final answer synthesis have different constraints and model requirements |
| Routing owner | Controller decides when to invoke context gathering | Small models should not be solely responsible for deciding whether tools or gatherers are needed |
| Gatherer contract | Return a typed evidence bundle instead of free-form prose | The synthesizer needs stable, inspectable inputs |
| Context-1 integration | Keep it experimental and capability-gated | The public model expects a specialized harness that Paddles does not yet have |
| Fallback behavior | Preserve the current path when gathering is unnecessary or unavailable | Protects the common case and keeps failure handling honest |

## Architecture

The runtime is split into five responsibilities:

- `IntentRouter`
  Classifies a turn as direct answer/tool work or retrieval-heavy work.
- `ContextGatherer`
  Executes retrieval and summarization for evidence collection.
- `EvidenceBundle`
  Carries ranked supporting items and gatherer metadata into synthesis.
- `Synthesizer`
  Produces the final user-facing answer using the default runtime.
- `Context1Adapter`
  Experimental gatherer provider that reports capability state explicitly.

## Components

- `IntentRouter`
  Purpose: choose the correct lane before prompt execution begins.
  Behavior: use heuristics or explicit controller signals to detect retrieval-
  heavy turns, while defaulting to the current direct-answer/tool lane.
- `ContextGatherer`
  Purpose: gather, rank, prune, and summarize evidence for a retrieval-heavy
  turn.
  Behavior: return evidence plus capability metadata; never return the final
  user answer as its primary contract.
- `EvidenceBundle`
  Purpose: provide a stable synthesis input.
  Behavior: contain ranked evidence items, aggregate summary, budget metadata,
  and warnings such as truncation or unsupported gatherer state.
- `Synthesizer`
  Purpose: preserve the current answer runtime as the final response generator.
  Behavior: combine the original user turn with any evidence bundle and produce
  the final answer/tool action.
- `Context1Adapter`
  Purpose: represent Chroma Context-1 without pretending its harness is native.
  Behavior: report `available`, `unsupported`, or `harness-required`, and only
  execute when the necessary environment is explicitly configured.

## Interfaces

The voyage introduces an internal contract of this shape:

```rust
struct ContextGatherRequest {
    user_query: String,
    workspace_root: PathBuf,
    evidence_budget: usize,
    intent_reason: String,
    prior_context: Vec<RetainedArtifactRef>,
}

enum GathererCapability {
    Available,
    Unsupported { reason: String },
    HarnessRequired { reason: String },
}

struct EvidenceItem {
    source: String,
    snippet: String,
    rank: usize,
    rationale: String,
}

struct EvidenceBundle {
    summary: String,
    items: Vec<EvidenceItem>,
    capability: GathererCapability,
    warnings: Vec<String>,
}
```

The exact Rust names can change, but the boundary must preserve this shape:
controller-owned request, typed capability reporting, and synthesis-ready
evidence output.

## Data Flow

1. A user turn enters the controller.
2. `IntentRouter` classifies the turn as direct/tool or retrieval-heavy.
3. For retrieval-heavy work, the controller builds a `ContextGatherRequest` and
   sends it to the configured gatherer lane.
4. The gatherer returns an `EvidenceBundle` or an explicit unsupported state.
5. The controller injects that evidence into the synthesizer lane.
6. The synthesizer produces the final response while the controller records
   routing and evidence metadata for debugging.
7. For direct/tool turns, the controller bypasses context gathering entirely and
   preserves the current path.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Retrieval-heavy turn is misclassified as direct/tool | Missing evidence in verbose/debug traces or failing routing proofs | Adjust router heuristics and preserve manual override options if needed | Retry with improved routing rules |
| Gatherer lane is unavailable | Capability check returns `unsupported` or `harness-required` | Fail closed and fall back to the existing path only when that path is still semantically correct | Operator can configure or disable the experimental gatherer |
| Gatherer returns too much or empty evidence | Bundle budget warnings or zero evidence items | Surface warnings in debug output and continue with bounded synthesis behavior | Tune evidence budget or gatherer implementation |
| Context-1 environment is configured incorrectly | Adapter initialization/runtime probe fails | Return explicit capability error rather than pretending the provider ran | Fix configuration or keep using the default gatherer path |
