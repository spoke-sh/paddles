# Paddles Architecture: The Mech Suit

This document describes the current architecture of `paddles`, the local-first agent harness for coding and workspace interaction.

## System Overview

`paddles` is a calibrated execution shell around a model runtime. It accepts user intent at the CLI, validates the boot contract, routes the turn through a local Sift-based controller, and returns either direct text or tool-backed results.

The architecture has two core responsibilities:

1. Preserve local-first operation for the default path.
2. Route each request to the smallest capable model and runtime for the job.

That routing principle is foundational. `paddles` should not assume one model is optimal for every turn. Chat, tool orchestration, workspace search, and long-horizon context gathering are different workloads and should be treated as such.

## Current Runtime Stack

### 1. CLI Entry (`src/main.rs`)
- Parses boot and runtime arguments.
- Supports one-shot prompts and interactive mode.
- Selects the requested model ID from the registry.

### 2. Boot and Application Layer (`src/application/mod.rs`, `src/domain/model/mod.rs`)
- Validates credits, weights, biases, and dogma.
- Initializes the application service.
- Owns the active engine instance used for prompt execution.

### 3. Model Registry (`src/infrastructure/adapters/sift_registry.rs`)
- Resolves logical model IDs such as `qwen-1.5b` into concrete asset locations.
- Ensures local model artifacts are present before inference begins.
- Acts as the boundary between `paddles` model names and backing runtimes.

### 4. Session Controller (`src/infrastructure/adapters/sift_agent.rs`)
- Interprets user intent for each turn.
- Distinguishes casual chat from action-oriented workspace requests.
- Assembles workspace context through Sift when retrieval is needed.
- Routes obvious workspace actions directly to tools when the controller can infer the correct call.
- Preserves short-turn session state such as recent turns, retained artifacts, and tool outputs.

### 5. Operator Memory Layer (`src/infrastructure/adapters/agent_memory.rs`)
- Reloads `AGENTS.md` memory on every prompt so the REPL can absorb operator guidance without restarting.
- Searches `/etc/paddles/AGENTS.md`, `~/.config/paddles/AGENTS.md`, and then every ancestor `AGENTS.md` from filesystem root to the current workspace.
- Treats later files as more specific than earlier files and injects the merged result into both direct-answer and tool-oriented prompt paths.

### 6. Local Generation Runtime (`src/infrastructure/adapters/sift_agent.rs`)
- Uses a reusable local Qwen runtime backed by `candle`.
- Keeps one loaded model runtime alive and resets turn state between sends.
- Executes the default local reasoning and tool-orchestration path.

### 7. Tool Surface
- Search and context assembly
- File listing and file reads
- File writes and replacements
- Shell execution
- Diff and patch application

The controller is responsible for deciding when tools are necessary and when a direct answer is sufficient.

## Data Flow

1. **Invoke**: The user runs `just paddles` or `paddles --prompt`.
2. **Calibrate**: `BootContext` validates the clean-boot invariants.
3. **Resolve Runtime**: The registry maps the requested model ID to local assets.
4. **Load Operator Memory**: The REPL reloads hierarchical `AGENTS.md` files and prepares a merged instruction block for the turn.
5. **Route Turn**: The Sift session controller classifies the turn:
   - Casual direct answer
   - Tool-backed workspace action
   - Context-assembled retrieval plus answer
6. **Execute**:
   - The local model replies directly, or
   - The controller executes tools and feeds the results back through the turn loop.
   - A retrieval-heavy request can gather evidence first, then hand that evidence to the synthesizer lane.
7. **Return**: The terminal prints the final response.

## Model Routing Strategy

Model routing should be driven by two inputs:

1. **User intent**
   - Casual chat
   - Deterministic workspace action
   - Code editing or tool orchestration
   - Multi-hop research or context gathering
   - Final synthesis or explanation
2. **Runtime constraints**
   - CPU vs CUDA
   - Available VRAM
   - Local-only vs remote-allowed
   - Latency budget
   - Corpus size and retrieval complexity

### Routing Principles

- Use the smallest capable local model for direct chat and straightforward tool orchestration.
- Prefer deterministic controller routing over asking a weak model to infer obvious shell or file actions.
- Keep retrieval and answer generation separate when the task is genuinely retrieval-heavy.
- Introduce a larger or specialized model only when the user's request actually needs it.
- Avoid paying a frontier-model tax for one-hop workspace lookups.

## Context-Gathering Subagents

Some tasks are dominated by retrieval rather than final answer generation. These include:

- Multi-hop discovery across many documents
- Long-horizon search where intermediate findings change the next query
- Tasks that accumulate too much context for a general assistant to manage cleanly
- Retrieval jobs where the best outcome is a ranked evidence set, not prose

These workloads justify a dedicated context-gathering model or subagent.

### Chroma Context-1 Fit

Chroma `context-1` is a strong candidate for this role, but only for a specific slice of the architecture.

- It is a 20B agentic search model trained to retrieve supporting documents rather than answer directly.
- It is intended to operate as a retrieval subagent alongside a separate reasoning model.
- It relies on a dedicated harness that manages tool execution, pruning, deduplication, and token budgets.
- It is not a drop-in replacement for the default `paddles` local answer model.

In `paddles`, a model like Context-1 belongs in the **context-gathering lane**, not the default conversational lane.

### Required Integration Shape

If adopted, a context-gathering model should sit behind an explicit controller boundary:

1. The router classifies the turn as retrieval-heavy.
2. The context subagent explores the corpus and returns ranked evidence.
3. A downstream answer model synthesizes the final response from that evidence.

This separation keeps search quality, search cost, and answer quality independently tunable.

### Typed Gatherer Contract

The gatherer boundary should be explicit in code, not implied by prompt text.
The current contract lives in `src/domain/ports/context_gathering.rs` and is
intended to preserve these semantics:

- `ContextGatherRequest`
  Carries the user query, workspace root, routing rationale, evidence budget,
  and prior context references.
- `ContextGatherResult`
  Returns explicit capability state plus an optional evidence bundle.
- `GathererCapability`
  Must distinguish `available`, `unsupported`, and `harness-required`.
- `EvidenceBundle`
  Carries a synthesis-ready summary, ranked evidence items, and warnings.

The important behavioral rule is simple: a gatherer returns evidence for a
downstream synthesizer. It does not return the final user-facing answer.

## Hierarchical Operator Memory

`paddles` now has an explicit operator-memory layer in the REPL runtime.

- Memory files are always named `AGENTS.md`.
- The runtime loads them in this order:
  1. `/etc/paddles/AGENTS.md`
  2. `~/.config/paddles/AGENTS.md`
  3. every ancestor `AGENTS.md` from filesystem root to the active workspace
- The loaded content is reapplied on every turn, not just at process start.
- More specific files override broader guidance by appearing later in the merged prompt.

This memory layer is guidance for prompt construction, not a replacement for
controller-side routing, typed evidence contracts, or deterministic tool
execution. The controller still owns routing, and memory must not be used as an
excuse to hide runtime behavior in prompt text.

### Experimental Context-1 Boundary

The current `context-1` integration is intentionally only a provider boundary.

- The controller can select `context-1` as a gatherer provider explicitly.
- The adapter reports `harness-required` until the external search harness is
  acknowledged as present.
- Even with that acknowledgement, the adapter reports `unsupported` until
  Paddles ships a real harness-backed provider implementation.
- In all non-available states, the controller falls back to the synthesizer
  lane instead of pretending the provider ran.

### Non-Goals

Context-gathering models should not become:

- The default path for simple chat
- The default path for obvious file or shell actions
- A hidden remote dependency in the local-first control plane
- A silent replacement for current local inference behavior

## Adoption Guidance

Before integrating any specialized retrieval model, verify:

1. The harness requirements are explicit and reproducible.
2. The runtime can support the model size and precision profile.
3. The controller can fall back to the local path when the specialized runtime is unavailable.
4. The output contract is evidence-first, not answer-first.
5. The routing logic is observable and testable.

## Current Direction

The near-term architectural direction for `paddles` is:

- Keep the default control plane local and lightweight.
- Improve controller-side intent routing.
- Add specialized subagents only behind explicit, testable routing boundaries.
- Treat context gathering, reasoning, and execution as separate responsibilities whenever that split improves quality, latency, or cost.
