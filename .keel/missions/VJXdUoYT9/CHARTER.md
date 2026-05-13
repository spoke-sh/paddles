# Unify Planner Action Schema Rendering - Charter

Archetype: Strategic

## Mission Intent

Eliminate planner action-schema drift by making the bounded planner action
schema one explicit authored contract rendered by every planner lane. The
planner must see the same action vocabulary, JSON examples, field requirements,
semantic-tool options, external capability shape, and completion constraints
whether the active planner is Sift/local, HTTP/remote, or a future lane.

This mission fixes a design flaw in the current architecture: the Rust enums
(`PlannerAction`, `InitialAction`, and `WorkspaceAction`) are shared, and the
dynamic execution contract is shared, but prompt-facing JSON action schemas are
duplicated in provider adapters. The result is avoidable drift between Sift,
HTTP, retry prompts, redecision prompts, semantic actions, and
`external_capability`.

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Add a single shared planner action schema renderer used by all planner lanes and all planner-action prompt variants. The renderer must produce the canonical JSON action examples, field requirements, and action-selection rules for initial action, recursive next action, retry, and redecision prompts. | board: VJXfKtEkv |
| MG-02 | Author the planner action schema as an explicit contract, with tests that prove it matches the Rust action enums. The contract must cover `InitialAction`, `PlannerAction`, `WorkspaceAction`, `answer`, `refine`, `branch`, `stop`, all workspace edit/read/probe actions, semantic workspace actions, and `external_capability`. | board: VJXfKtWku |
| MG-03 | Remove adapter-local hardcoded action-schema blocks from Sift and HTTP planner prompts. Provider-specific prompt text may still describe transport mechanics, but action names, JSON examples, required fields, and shared rules must come from the shared renderer. | board: VJXfKtukt |
| MG-04 | Prove end-to-end mocked turns for Sift and HTTP receive the same canonical planner action schema. Tests must cover initial planner prompts and recursive next-action prompts, and must fail if either lane drifts from the shared schema. | board: VJXfKuNks |
| MG-05 | Update foundational documentation so README, POLICY, ARCHITECTURE, and any owning docs describe one shared planner action schema contract instead of adapter-local prompt schemas. | board: VJXfKufl7 |

## Constraints

- **Single source of truth.** Do not introduce another adapter-local action list,
  schema block, or JSON example list. Any new planner lane must call the shared
  renderer.
- **Authored contract, not enum reflection magic.** The shared schema should be
  explicit enough to review as product/API surface. Tests enforce parity with
  the Rust enums.
- **All current action surface included.** Include semantic workspace actions
  and `external_capability` now; do not leave them as future follow-up drift.
- **Transport text stays adapter-specific.** HTTP can still explain native tool
  calls, structured JSON, or prompt envelopes; Sift can still tune local-model
  instructions. They must not own the action schema itself.
- **Runtime capability filtering still applies.** The shared schema describes
  the contract; the existing `PlannerExecutionContract` and capability manifest
  still disclose what is available, blocked, or governed in the active turn.
- **Foundational docs move with behavior.** README, POLICY, ARCHITECTURE, and
  any owning docs touched by this contract must update in the same mission.
- **No semantic broadening.** This mission unifies and verifies the existing
  action surface. It does not add new planner actions except where an existing
  enum variant is already present but missing from prompt-facing schema.

## Halting Rules

- DO NOT halt while any MG-* goal still depends on adapter-local action-schema
  text, missing enum/schema parity tests, missing Sift/HTTP mocked-turn proof,
  or stale foundational documentation.
- HALT when one shared planner action schema renderer is used by every planner
  prompt path, tests prove parity with Rust enums, mocked Sift and HTTP turns
  receive the same canonical schema, and foundational docs describe that
  contract.
- YIELD to the human if the implementation would require changing the planner /
  response-lane architectural split, removing current planner actions, or
  weakening local-first execution governance.
