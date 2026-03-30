# Paddles Policy: Operational Invariants

> The operational invariants and constraints that govern the Paddles Mech Suit.
>
> Foundational stack position: `5/8`
> Read this after [CONSTITUTION.md](CONSTITUTION.md) and before [ARCHITECTURE.md](ARCHITECTURE.md).

## 1. Boot Invariants (The Clean Boot)
A boot sequence is only valid (CLEAN) if:
- **Credits**: Initial balance is non-negative (default 0).
- **Weights**: Calibration values are within the `min_weight` and `max_weight` defined in the `Constitution`.
- **Dogma**: The session does not trigger `reality_mode` or equivalent dogma violations.

## 2. Zero Drift Policy
Progress is blocked if structural or requirement drift is detected:
- **Requirement Drift**: Stories missing SRS/AC links or verification proofs.
- **Structural Drift**: Broken board integrity or orphaned entities.
- **Scaffold Drift**: Unfilled template placeholders (`{{goal}}`).

## 3. Local Capacity Invariant
Features should be implemented using local inference (`candle`) or local-first tools whenever possible. New network dependencies must be justified via an ADR.

## 4. Specialized Model Routing
Model selection is an architectural decision, not an incidental prompt tweak.
- **Smallest Capable Model**: Route each request to the smallest model that can satisfy the user's intent within the active runtime constraints.
- **Intent Before Size**: Choose models based on workload shape first: direct answer, tool orchestration, retrieval, context gathering, or final synthesis.
- **Interpretation Before Routing**: Operator memory and foundational guidance should be available to first-pass planner interpretation before the runtime commits to a route.
- **Model-Directed Top-Level Action Selection**: After interpretation context is assembled, the primary mech-suit path should choose its next bounded action through a constrained model response rather than controller string heuristics.
- **Controller Owns Validation, Not Heuristic Intent**: The controller remains authoritative for schema validation, budgets, safe command allowlists, deterministic execution, and fail-closed behavior, but not for guessing the user's next resource action through ad hoc string rules.
- **Recursive Planning Before Final Synthesis**: Difficult workspace questions should be improved through bounded recursive resource use rather than relying solely on one-shot controller heuristics.
- **Planner Action Contract**: Turns should flow through a constrained planner contract that can express direct answer or synthesize, concrete workspace actions (`search`, `list_files`, `read`, `inspect`, `shell`, `diff`, `write_file`, `replace_in_file`, `apply_patch`), refine, branch, and stop decisions.
- **Separate Catalog From Runtime From Prompting**: Model integration must isolate alias/asset resolution, family-specific inference runtime, and prompt/protocol behavior so new families can be added without cross-cutting regressions.
- **Separate Planner From Synthesizer**: Recursive planning or context gathering may use a dedicated planner-capable model, but final answer generation should remain a distinct step.
- **Evidence-First Gatherers**: Context-gathering adapters must return typed evidence bundles and capability state for a downstream synthesizer, not prose pretending to be the final answer.
- **Graph Retrieval Stays Behind The Gatherer Boundary**: Richer graph/branching retrieval should strengthen the generic gatherer path and typed evidence contract, not introduce repository-specific top-level runtime intents.
- **Turns Plan First**: The primary mech-suit runtime should ask the configured planner lane for the first bounded action before route selection. Hidden synthesizer-private retrieval is a fallback/debug path, not the normal repository-answer path.
- **Grounded Answers Cite Sources**: Repository-question answers must include file citations by default and should degrade to extractive evidence or explicit insufficiency rather than unsupported prose.
- **Planner Metadata Must Stay Observable**: When a gatherer uses autonomous planning, planner strategy, stop reason, and retained-evidence summaries must remain observable in verbose output or evidence digests.
- **Graph Lineage Must Stay Typed**: Branch ids, frontier ids, node ids, edge kinds, and graph stop state should remain in `paddles`-owned structured metadata so future embedded recorders do not need to reconstruct lineage from UI prose.
- **Interactive Sessions Own A Durable Conversation Root**: Multi-turn interactive work should reuse one paddles-owned task root with stable turn, branch, candidate, and decision ids rather than minting unrelated prompt-local traces.
- **Steering Prompts Become Structured Thread Candidates**: A prompt captured during an active turn must be preserved as a typed candidate with stable lineage, not an opaque queue string.
- **Thread Selection Is Model-Directed**: Continue-current-thread, open-child-thread, and merge-back decisions should come from a constrained model response using interpretation context and known thread state.
- **Merge-Back Must Stay Explicit**: Reconciliation into the mainline or another thread must be recorded as an explicit outcome with replayable lineage and transcript visibility, never as hidden history rewrite.
- **Trace Recording Is Separate From Rendering**: Transcript rows and default TUI events remain operator-facing projections. Durable turn lineage must flow through the `paddles`-owned trace contract and `TraceRecorder` boundary.
- **Storage Neutral Domain Contract**: `paddles` may use embedded `transit-core` as the first durable recorder backend, but raw transit kernel or server types must not leak across the domain boundary.
- **Artifact Envelopes Before Blob Sprawl**: Large prompts, tool outputs, model outputs, and graph traces should be represented through artifact envelopes with stable logical ids plus inline metadata instead of assuming inline-only storage forever.
- **Recorder Failures Must Fail Closed**: Recorder unavailability or storage failure must not corrupt turn execution. The runtime should degrade to noop recording and surface the loss of durability explicitly.
- **Turn Events Stay Visible**: The interactive REPL must render a default action stream for classification, route selection, gatherer work, tool calls, fallbacks, and synthesis. This is not reserved for a quiet debug mode.
- **Thread Events Stay Visible**: The default transcript should show steering capture, thread split/continue decisions, active-thread changes, and merge-back outcomes without requiring a debug-only renderer.
- **TTY UI / Plain CLI Split**: Interactive terminal sessions should use the dedicated transcript UI, while `--prompt` and non-TTY stdin/stdout flows must remain plain output paths for scripting and pipes.
- **Hierarchical Operator Memory**: The REPL must reload `AGENTS.md` memory on every turn from system, user, and ancestor scopes in that order, with more specific files overriding broader guidance.
- **Memory Shapes Action Selection; Control Shapes Safety**: Instruction memory rooted at `AGENTS.md` and its turn-time model-derived guidance graph should influence first action selection, while typed evidence contracts, deterministic execution, validation, and budgets remain controller-owned.
- **AGENTS.md Is The Interpretation Root**: The harness may hardcode discovery of `AGENTS.md` memory files, but it should not hardcode a named foundational document set beyond that root.
- **The AGENTS Guidance Graph Can Bootstrap Tool Hints**: The turn-time guidance graph derived from `AGENTS.md` may contribute read-only command hints into interpretation context so small local planner models can discover better next actions without hardcoded repository intents.
- **The AGENTS Guidance Graph Can Bootstrap Decision Procedures**: The same model-derived memory graph may contribute ordered, prompt-relevant procedures so fallback planning and stop decisions can be shaped by project guidance rather than repository-specific controller heuristics.
- **Keel Is Context, Not A Built-In Intent**: Project or board engines may inform planning through recursive context and tools, but the harness should not overfit by encoding one engine as a product-specific first-class turn type.
- **Explicit Harness Requirement**: A specialized retrieval model must not be treated as a drop-in answer model when it depends on a custom tool harness, pruning loop, or context manager.
- **No Silent Remote Regression**: The default `paddles` path remains local-first. Remote or heavyweight specialized models must be explicit, observable, and degradable to a local fallback.
- **No Pretend Concurrency**: Auto-threading may queue, branch, replay, and merge work, but it must not imply simultaneous sibling generation on one local runtime unless that capability is actually implemented.

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
