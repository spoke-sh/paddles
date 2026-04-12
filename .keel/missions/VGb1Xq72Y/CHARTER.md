# Govern Recursive Execution With Sandboxed Hands - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Equip Paddles' recursive harness with explicit sandbox posture, approval-policy evaluation, and bounded privilege-escalation semantics across execution hands so tool use can be trusted, replayed, and audited instead of relying on raw local shell access. | board: VGb1c0pAN |

## Constraints

- Keep the execution model local-first. Hardening should come from bounded local mediation and policy, not from assuming a hosted control plane.
- Apply one shared governance vocabulary across shell, workspace-edit, transport, and future external hands so permission behavior does not fork by surface.
- Denied, deferred, escalated, and approved actions must become traceable runtime events and operator-visible transcript states rather than hidden controller branches.
- Profile negotiation may downgrade or disable privileged hands, but it must do so explicitly and honestly.

## Halting Rules

- DO NOT halt while raw command execution can bypass the declared sandbox or approval posture for the active turn.
- DO NOT halt while operators cannot tell why an action was allowed, denied, or escalated.
- DO NOT halt while privileged execution paths fail open when policy state or capability data is incomplete.
- HALT when epic `VGb1c0pAN` is terminal and the execution-governance model is covered by docs, tests, and proof artifacts.
- YIELD to human only if the default trust posture or approval UX requires a product decision rather than an implementation decision.
