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
- **Separate Catalog From Runtime From Prompting**: Model integration must isolate alias/asset resolution, family-specific inference runtime, and prompt/protocol behavior so new families can be added without cross-cutting regressions.
- **Separate Search From Answering**: Retrieval-heavy or multi-hop tasks may use a dedicated context-gathering model, but answer generation should remain a distinct step.
- **Evidence-First Gatherers**: Context-gathering adapters must return typed evidence bundles and capability state for a downstream synthesizer, not prose pretending to be the final answer.
- **Hierarchical Operator Memory**: The REPL must reload `AGENTS.md` memory on every turn from system, user, and ancestor scopes in that order, with more specific files overriding broader guidance.
- **Memory Does Not Replace Control**: Instruction memory may shape prompt construction, but controller-owned routing, typed evidence contracts, and deterministic tool execution remain authoritative.
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
