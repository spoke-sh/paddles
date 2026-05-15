# Collapse Pre-Loop Routing Into Agent Loop - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Make the recursive agent loop the first and only model-owned action-selection path for normal turns. Remove the separate pre-loop initial action/router phase, move turn policy and mutation/output posture into the loop/execution contract, and let the first loop iteration choose answer, evidence, edit, commit, or stop actions through the same bounded action-selection protocol as every later step. | board: VJeQx1O20 |

## Constraints
- Preserve execution governance and capability-manifest enforcement; this mission removes duplicate pre-loop routing, not safety boundaries.
- Keep provider wire compatibility only where required by existing HTTP contracts; runtime vocabulary should reflect agent-loop ownership.
- Prove the migration with focused loop/routing tests plus full library verification.
## Halting Rules

- DO NOT halt while MG-01 has unfinished board work.
- HALT when epic VJeQx1O20 and its scoped voyages/stories are complete and verified.
