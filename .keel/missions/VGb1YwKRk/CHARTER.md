# Build An Interruptible Turn And Thread Control Plane - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Give Paddles a first-class turn and thread control plane with same-turn steering, interruption, fork/resume semantics, and live plan/diff state so the recursive harness can be steered as a runtime instead of only as a prompt loop. | board: VGb1c1AAK |

## Constraints

- Build on the existing recorder, replay, and thread-lineage work instead of introducing a parallel session substrate.
- Keep one recursive core loop. The control plane should expose and steer that loop, not replace it with a different product metaphor.
- Use typed runtime items and events for plans, diffs, command/file changes, and control transitions instead of relying on opaque string rendering.
- The resulting control surface must be consumable by TUI, web, and HTTP/API layers without each surface inventing different turn semantics.

## Halting Rules

- DO NOT halt while active-turn steering still degrades into queued follow-up input without true interruptible turn semantics.
- DO NOT halt while fork/resume/rollback or equivalent thread lifecycle transitions remain implicit or non-replayable.
- DO NOT halt while plan and diff state needed to understand a live turn are unavailable to operator-facing surfaces.
- HALT when epic `VGb1c1AAK` is terminal and the control plane is proven through runtime tests, transcript proofs, and updated docs.
- YIELD to human only if the user-facing semantics of interruption, rollback, or turn ownership require a product decision.
