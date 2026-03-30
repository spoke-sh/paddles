# Step Timing Baselines - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Per-event-type step timing reservoir with pace classification and colored transcript rendering | board: VFNccFj7d |

## Constraints

- Storage lives in ~/.cache/paddles/, not ~/.config/paddles/
- Reservoir is loaded at boot, flushed after each turn completes
- Classification keys derived from TurnEvent serde tag names
- Steps with insufficient history render as normal (no guessing)
- No new crate dependencies for this feature

## Halting Rules

- HALT when epic VFNccFj7d is verified
- YIELD to human for color/threshold tuning after first integration
