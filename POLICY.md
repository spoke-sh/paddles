# Paddles Policy: Operational Invariants

> The operational invariants and constraints that govern the Paddles Mech Suit.

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
- **Planner Action Contract**: Turns should flow through a constrained planner contract that can express direct answer or synthesize, search, read, inspect, refine, branch, and stop decisions. A temporary `tool` bridge action is allowed while deterministic tool execution remains a separate runtime.
- **Separate Catalog From Runtime From Prompting**: Model integration must isolate alias/asset resolution, family-specific inference runtime, and prompt/protocol behavior so new families can be added without cross-cutting regressions.
- **Separate Planner From Synthesizer**: Recursive planning or context gathering may use a dedicated planner-capable model, but final answer generation should remain a distinct step.
- **Evidence-First Gatherers**: Context-gathering adapters must return typed evidence bundles and capability state for a downstream synthesizer, not prose pretending to be the final answer.
- **Turns Plan First**: The primary mech-suit runtime should ask the configured planner lane for the first bounded action before route selection. Hidden synthesizer-private retrieval is a fallback/debug path, not the normal repository-answer path.
- **Grounded Answers Cite Sources**: Repository-question answers must include file citations by default and should degrade to extractive evidence or explicit insufficiency rather than unsupported prose.
- **Planner Metadata Must Stay Observable**: When a gatherer uses autonomous planning, planner strategy, stop reason, and retained-evidence summaries must remain observable in verbose output or evidence digests.
- **Turn Events Stay Visible**: The interactive REPL must render a default action stream for classification, route selection, gatherer work, tool calls, fallbacks, and synthesis. This is not reserved for a quiet debug mode.
- **TTY UI / Plain CLI Split**: Interactive terminal sessions should use the dedicated transcript UI, while `--prompt` and non-TTY stdin/stdout flows must remain plain output paths for scripting and pipes.
- **Hierarchical Operator Memory**: The REPL must reload `AGENTS.md` memory on every turn from system, user, and ancestor scopes in that order, with more specific files overriding broader guidance.
- **Memory Shapes Action Selection; Control Shapes Safety**: Instruction memory and linked foundational docs should influence first action selection, while typed evidence contracts, deterministic execution, validation, and budgets remain controller-owned.
- **Keel Is Context, Not A Built-In Intent**: Project or board engines may inform planning through recursive context and tools, but the harness should not overfit by encoding one engine as a product-specific first-class turn type.
- **Explicit Harness Requirement**: A specialized retrieval model must not be treated as a drop-in answer model when it depends on a custom tool harness, pruning loop, or context manager.
- **No Silent Remote Regression**: The default `paddles` path remains local-first. Remote or heavyweight specialized models must be explicit, observable, and degradable to a local fallback.

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
