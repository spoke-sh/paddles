# Paddles Policy: Operational Commitments

> The guarantees and operating principles that govern the Paddles Mech Suit.
>
> Foundational stack position: `5/8`
> Read this after [CONSTITUTION.md](CONSTITUTION.md) and before [ARCHITECTURE.md](ARCHITECTURE.md).

## 1. Boot Invariants (The Clean Boot)
A boot sequence is only valid (CLEAN) if:
- **Credits**: Initial balance is non-negative (default 0).
- **Weights**: Calibration values are within the `min_weight` and `max_weight` defined in the `Constitution`.
- **Dogma**: The session remains within structured simulation bounds — free of `reality_mode` or equivalent dogma violations.

## 2. Zero Drift Policy
Progress is blocked if structural or requirement drift is detected:
- **Requirement Drift**: Stories missing SRS/AC links or verification proofs.
- **Structural Drift**: Broken board integrity or orphaned entities.
- **Scaffold Drift**: Unfilled template placeholders (`{{goal}}`).

## 3. Local Capacity Invariant
Features should be implemented using local inference (`candle`) or local-first tools whenever possible. New network dependencies must be justified via an ADR.

## 4. Specialized Model Routing
Model selection is an architectural decision that shapes every turn.

### Routing Principles
- **Smallest Capable Model**: Each request routes to the smallest model that satisfies the workload within active runtime constraints.
- **Intent Before Size**: Models are chosen by workload shape first — direct answer, tool orchestration, retrieval, context gathering, or final synthesis.
- **Interpretation Before Routing**: Operator memory and foundational guidance are available to the planner before any routing commitment.
- **Model-Directed Action Selection**: The planner selects its next bounded action through a constrained model response after interpretation context is assembled.
- **Controller Owns Validation and Safety**: The controller remains authoritative for schema validation, budgets, safe command allowlists, deterministic execution, and fail-closed behavior. The model drives direction; the controller ensures safety.
- **Intent Lives In Model Decisions, Not Controller Heuristics**: The controller must not infer workspace engagement or direct-answer intent from prompt-token heuristics. Intent routing comes from the planner's typed decision.

### Planning and Synthesis
- **Recursive Planning Earns Better Answers**: Difficult workspace questions improve through bounded recursive resource use — each iteration adds real evidence.
- **Planner Action Contract**: Turns flow through a constrained planner contract: answer/synthesize, workspace actions (`search`, `list_files`, `read`, `inspect`, `shell`, `diff`, `write_file`, `replace_in_file`, `apply_patch`), refine, branch, and stop.
- **Turns Plan First**: The primary mech-suit runtime asks the planner for its first bounded action before route selection. This recursive path is the primary mode of operation.
- **Separate Planner From Synthesizer**: Planning and final answer generation are distinct steps, each routed to the model best suited for that workload.
- **Grounded Answers Cite Sources**: Repository-question answers include file citations by default and degrade to extractive evidence or explicit insufficiency when sources are unavailable.
- **Final Answer Rendering Stays Typed**: Planner-direct answers and synthesizer answers both normalize through the same canonical render AST (`heading`, `paragraph`, `bullet_list`, `code_block`, `citations`); operators see a normalized transcript projection instead of raw markdown conventions.
- **Planner Rationale Is Never The User Answer**: Planner rationale explains control decisions. User-facing answer text must travel through an explicit answer payload before it enters the render pipeline.
- **Planner And Answer Lanes Share One Conversational Handoff**: Recent turns and active-thread summaries are carried through a typed handoff into the answer lane, so follow-up turns remain coherent across planner-direct and synthesizer-authored replies.
- **Generative Authoring Stays Separate From Rendering**: Rich surface-aware expression belongs in a generative authoring layer that targets the canonical render AST and surface affordances. Renderers project that typed output; they do not invent or reinterpret content.

### Steering Signals
- **Steering Signals Are Typed Controller Policies**: Steering signals are not hidden vibe checks. They are the family of explicit controller policies that bias or stop recursive work as evidence accumulates.
- **Context Strain Stays Observable**: Truncation and evidence-budget loss surface as `ContextStrain` events and influence snapshots. They make context degradation visible without silently changing the answer contract.
- **Action Bias Prefers Action Over Drift**: Mutation turns should move toward a plausible file read or edit once enough evidence exists. Repeated non-file probing is a controller-corrected failure mode.
- **Premise Challenge Revises Priors**: User reports start as hypotheses, not facts. When gathered sources weaken a premise, the harness should stop redundant confirmatory probes and yield a source-judged answer.
- **Compaction Cue Preserves Actionability**: Context refinement and compaction may summarize or prune low-value artifacts when depth threatens actionability, while preserving locators to deeper records.
- **Budget Boundary Ends The Loop Honestly**: Step, search, inspect, and read caps are hard limits. When the budget is spent, the harness stops and answers from the evidence it actually has.
- **Influence Snapshots Stay Visible**: Steering signals must remain legible in transit traces with source-attributed contributions, so operators can inspect what shaped a turn.

### Evidence and Gathering
- **Evidence-First Gatherers**: Context-gathering adapters return typed evidence bundles and capability state for downstream synthesis.
- **Graph Retrieval Strengthens The Gatherer Path**: Richer graph/branching retrieval enhances the generic gatherer boundary and typed evidence contract, keeping the harness general-purpose.
- **Planner Metadata Stays Observable**: Planner strategy, stop reason, and retained-evidence summaries remain visible in verbose output and evidence digests.
- **Graph Lineage Stays Typed**: Branch ids, frontier ids, node ids, edge kinds, and graph stop state live in `paddles`-owned structured metadata, ready for embedded recorders and replay.

### Model Integration
- **Separate Catalog, Runtime, and Prompting**: Model integration isolates alias/asset resolution, family-specific inference runtime, and prompt/protocol behavior — new families integrate cleanly.
- **Specialized Models Require Their Harness**: A specialized retrieval model runs within its proper harness, pruning loop, or context manager — recognized as a distinct capability rather than a drop-in substitute.
- **Local-First By Default**: The default path runs on local models. Remote or heavyweight models are explicit, observable, and degrade gracefully to local fallbacks.

### Interactive Sessions and Threading
- **Durable Conversation Root**: Multi-turn interactive work reuses one paddles-owned task root with stable turn, branch, candidate, and decision ids.
- **Structured Thread Candidates**: Prompts captured during an active turn are preserved as typed candidates with stable lineage.
- **Model-Directed Thread Selection**: Continue, branch, and merge-back decisions come from a constrained model response using interpretation context and known thread state.
- **Explicit Merge-Back Records**: Reconciliation into the mainline or another thread is recorded as an explicit outcome with replayable lineage and transcript visibility.
- **Honest Concurrency**: Auto-threading queues, branches, replays, and merges work. Simultaneous sibling generation ships when it is actually implemented.

### Trace Recording and Visibility
- **Recording Is Separate From Rendering**: Durable turn lineage flows through the `paddles`-owned trace contract and `TraceRecorder` boundary. Transcript rows remain operator-facing projections.
- **Storage Neutral Domain Contract**: `paddles` uses embedded `transit-core` as its first durable recorder backend, with transit types staying behind the domain boundary.
- **Artifact Envelopes For Large Payloads**: Large prompts, tool outputs, model outputs, and graph traces use artifact envelopes with stable logical ids and inline metadata, supporting future external storage.
- **Recorder Failures Degrade Safely**: Recorder unavailability degrades to noop recording with explicit notification — turn execution stays healthy.
- **Turn Events Stay Visible**: The interactive REPL renders a default action stream for classification, route selection, gatherer work, tool calls, fallbacks, and synthesis.
- **Thread Events Stay Visible**: The default transcript shows steering capture, thread decisions, active-thread changes, and merge-back outcomes.
- **TTY and Plain CLI Serve Different Needs**: Interactive sessions use the dedicated transcript UI; `--prompt` and non-TTY flows remain plain output paths for scripting and pipes.

### Operator Memory
- **Hierarchical Memory**: The REPL reloads `AGENTS.md` memory on every turn from system, user, and ancestor scopes, with more specific files taking precedence.
- **Memory Drives Action, Control Drives Safety**: Instruction memory rooted at `AGENTS.md` and its model-derived guidance graph influence action selection. Typed evidence contracts, validation, and budgets remain controller-owned.
- **AGENTS.md Is The Interpretation Root**: The harness discovers `AGENTS.md` memory files; additional guidance flows through the model-derived graph — keeping the system extensible.
- **Guidance Graph Bootstraps Tool Hints**: The turn-time guidance graph contributes read-only command hints so small local planner models discover better next actions through project knowledge.
- **Guidance Graph Bootstraps Decision Procedures**: The same model-derived graph contributes ordered, prompt-relevant procedures so fallback planning and stop decisions are shaped by project guidance.
- **Project Engines Are Context**: Keel and other board engines inform planning through recursive context and tools — the harness stays general-purpose and useful across domains.

## 5. Entity State Machine
Follow strict transition gates for all `.keel/` entities:
- **Planned**: Requires SRS/SDD authored content.
- **Started**: Requires an active parent Voyage.
- **Submitted**: Requires recorded proof for every Acceptance Criterion.
- **Verified**: Requires human sign-off of the submitted evidence.

## 6. Pacemaker Synchronization
Every commit that modifies the board MUST be pace-set by `keel poke` and include the `keel doctor --status` Importance Snapshot in the commit message.

## 7. Mission Achievement
A mission is **Achieved** only when:
- **Goals Met**: All `board:` goals are satisfied by terminal child entities.
- **Work Closed**: No open implementation work remains in the mission's scope.
- **Log Sealed**: A final session digest is recorded in `LOG.md`.
