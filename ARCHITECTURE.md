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
- Supports one-shot prompts, non-TTY plain interactive input, and a TTY-only alternate-screen interactive TUI.
- Selects the requested model ID from the registry.
- Chooses the frontend path explicitly: plain stdout for `--prompt` and non-TTY flows, transcript TUI for interactive terminals.

### 2. Boot and Application Layer (`src/application/mod.rs`, `src/domain/model/mod.rs`)
- Validates credits, weights, biases, and dogma.
- Initializes the application service.
- Owns the active engine instance used for prompt execution.

### 3. Model Registry (`src/infrastructure/adapters/sift_registry.rs`)
- Resolves logical model IDs such as `qwen-1.5b`, `qwen-coder-1.5b`, `qwen-coder-3b`, and `qwen3.5-2b` into concrete asset locations.
- Ensures local model artifacts are present before inference begins.
- Acts as the catalog boundary between `paddles` model names and backing runtimes.

### 3a. Model Integration Layers

Model-family complexity is intentionally split into three local layers so adding
support for a new family does not turn the registry into a god object:

1. **Catalog / Registry**
   Maps logical IDs to a typed model spec: backing repo, revision, family,
   weight layout, and local asset paths.
2. **Backend Runtime**
   Owns family-specific config parsing, `candle` model construction, weight
   dtype policy, cache reset, and token generation.
3. **Prompt / Protocol**
   Owns the prompt wrapper and response protocol that `paddles` expects for a
   family. This is controlled by `paddles`, not delegated blindly to an
   upstream chat template.

That split is what makes `Qwen3.5` support additive to existing `Qwen2`
support instead of a cross-cutting rewrite.

### 4. Session Controller (`src/infrastructure/adapters/sift_agent.rs`)
- Interprets user intent for each turn.
- Distinguishes casual chat, deterministic actions, repository questions, decomposition/research turns, and general questions.
- Routes repository questions through the explicit gatherer boundary by default instead of treating synthesizer-private retrieval as the normal path.
- Routes obvious workspace actions directly to tools when the controller can infer the correct call.
- Preserves short-turn session state such as recent turns, retained artifacts, and tool outputs.

### 4a. Turn Event Stream (`src/domain/model/turns.rs`, `src/application/mod.rs`)
- Emits typed turn events for intent classification, route selection, gatherer capability, gatherer summaries, planner summaries, tool execution, fallbacks, context assembly, and synthesis readiness.
- Feeds those events into the default interactive TUI as Codex-style action rows.
- Exists to make runtime behavior inspectable without requiring debug-only backend logs.

### 4b. Interactive Terminal UI (`src/infrastructure/cli/interactive_tui.rs`)
- Owns terminal raw-mode and alternate-screen lifecycle for TTY interactive sessions.
- Maintains transcript state for user rows, assistant rows, error rows, and action/event rows.
- Styles the transcript with distinct user/assistant/action palettes that adapt to light or dark terminals.
- Progressively reveals the finalized assistant answer after the turn completes while preserving the grounded/cited final content.

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
5. **Route Turn**: The application controller classifies the turn:
   - Casual direct answer
   - Deterministic workspace action
   - Repository question
   - Decomposition/research turn
   - General question
6. **Execute**:
   - The local model replies directly, or
   - The controller executes tools and feeds the results back through the turn loop.
   - A repository or decomposition request gathers evidence first, then hands that evidence to the synthesizer lane.
7. **Render**:
   - TTY interactive sessions render the Codex-style transcript TUI with event rows and progressive assistant output.
   - `--prompt` and non-TTY flows stay on the plain stdout path.
8. **Return**: The final response preserves source citations for repository-question turns.

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
- Prefer `qwen-1.5b` as the default local response lane for general interactive use, keep `qwen-coder-0.5b` and `qwen-coder-1.5b` available for coding-biased turns, expose `qwen-coder-3b` and `qwen3.5-2b` as explicit heavier options, and fail over to CPU when the Qwen3.5 CUDA runtime cannot load or generate safely.
- Prefer deterministic controller routing over asking a weak model to infer obvious shell or file actions.
- Keep retrieval and answer generation separate when the task is genuinely retrieval-heavy.
- Treat repository questions as evidence-first by default: gather, synthesize, and cite.
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
  Carries a synthesis-ready summary, ranked evidence items, warnings, and
  optional planner metadata for autonomous gatherers.
- `PlannerTraceMetadata`
  Carries planner strategy, step/decision trace, stop reason, and retained
  artifact summaries when a gatherer used bounded autonomous planning.

The important behavioral rule is simple: a gatherer returns evidence for a
downstream synthesizer. It does not return the final user-facing answer.

### Local Sift Autonomous Gatherer

`paddles` now also exposes a local `sift-autonomous` gatherer provider.

- It uses `Sift::search_autonomous` behind the typed gatherer contract.
- It defaults to Sift's heuristic planner strategy.
- It returns ranked evidence plus planner trace metadata, stop reason, and
  retained artifact summaries.
- It is intended for repository investigation and decomposition-heavy retrieval,
  not for casual chat or deterministic workspace actions.
- It still feeds the normal synthesizer lane rather than bypassing the final
  answer model.

Repository questions now prefer this explicit gatherer path by default. If the
gatherer is unavailable or reports failure, the controller emits a labeled
fallback event and degrades honestly instead of implying that gatherer-backed
evidence was used.

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

## Grounded Synthesis Contract

Repository-question synthesis is constrained by an evidence-first contract:

- The synthesizer should consume explicit evidence bundles, not improvise from
  private context assembly.
- Final repository answers should include file citations by default.
- If the model reply is empty, ungrounded, or cannot be tied to gathered
  evidence, the runtime should fall back to an extractive evidence summary.
- If no usable evidence exists, `paddles` should say so explicitly instead of
  inventing repository facts.

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
