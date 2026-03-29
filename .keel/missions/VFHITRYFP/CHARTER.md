# Implement Model-Driven Auto-Threading With Transit - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Add model-driven auto-threading to `paddles` so steering prompts that arrive during an active turn can be classified by the model as either continuing the current thread or opening a child branch, with explicit transit-backed lineage, replayable artifacts, and an operator-visible merge-back experience. | board: VFHIUOcFc |

## Constraints

- Keep thread selection model-driven. The model is the classifier for whether a steering prompt continues the current thread, opens a child branch, or asks for later merge/reconciliation; the controller may validate and bound the result but should not replace it with hardcoded domain-specific heuristics.
- Keep the capability generic. Do not introduce Keel-first, board-first, or other repository-specific thread intents. Keel is one evidence domain inside the workspace, not a special top-level runtime mode.
- Build on the existing `TraceRecorder` boundary and embedded `transit-core` path, but keep the conversation/thread API in a paddles-owned layer or crate first so it can be extracted later without expanding `transit-core` into an application-specific conversation surface. The mission must not require a networked `transit` server for normal local auto-threading.
- Preserve explicit, replayable thread lineage. Thread creation, replies, backlinks, summaries, merge decisions, and completion checkpoints must be durable artifacts or records rather than hidden UI state.
- Preserve interactive control. The operator must be able to keep typing while a turn is active, and the runtime must treat steering prompts as first-class thread candidates rather than dropping them or holding them as opaque in-memory strings.
- Preserve local-first bounded execution. Auto-threading must degrade honestly when the planner model, recorder, or thread merge path is unavailable, invalid, or over budget.
- Merge-back must not rewrite history. Returning thread work to the mainline should happen through explicit merge, backlink, or summary records with a legible transcript UX.
- Foundational docs must explain root-thread vs child-thread behavior, merge-back semantics, and the remaining concurrency limitations honestly.

## Halting Rules

- DO NOT halt while epic `VFHIUOcFc` or its child voyage/stories still contain unplanned or unfinished work.
- HALT when epic `VFHIUOcFc` is verified and `paddles` can route steering prompts through model-driven thread decisions with explicit transit-backed lineage and a thread-aware UX.
- YIELD if the design would require hardcoded product-specific thread heuristics, a mandatory networked trace server, or hidden mutation of thread structure that cannot be replayed from the recorded artifacts.
