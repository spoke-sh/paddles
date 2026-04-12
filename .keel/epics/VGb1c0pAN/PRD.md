# Sandboxed Execution And Approval Policies - Product Requirements

## Problem Statement

Paddles executes shell and local hands with only light validation, so the recursive harness cannot express Codex-style sandbox posture, per-action approval requirements, or bounded privilege escalation.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Define one execution-governance model for recursive hands that separates sandbox posture from approval policy. | The runtime selects an explicit sandbox mode and approval policy for each turn and records that choice. | Governance is visible before the first side-effecting action runs. |
| GOAL-02 | Enforce permission checks across shell, workspace-edit, transport, and future external hands through a shared gate. | Side-effecting actions either execute under policy, request escalation, or fail closed with explicit diagnostics. | No privileged path bypasses the shared gate. |
| GOAL-03 | Make execution governance legible to operators and replay surfaces. | Transcript, trace, and web/runtime projections show allow/deny/escalate outcomes and rationale. | Operators can explain why a command ran or did not run. |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Local Operator | A developer using Paddles interactively on a real repository. | Confidence that the harness will not overreach without an explicit policy or approval path. |
| Runtime Maintainer | A contributor hardening Paddles' recursive execution model. | One reusable governance layer instead of bespoke command filtering at each tool surface. |
| Surface Integrator | A maintainer exposing Paddles through TUI, web, or API surfaces. | A stable execution-policy contract that can be rendered consistently across interfaces. |

## Scope

### In Scope

- [SCOPE-01] Define explicit sandbox modes and approval policies for recursive execution hands.
- [SCOPE-02] Introduce a shared permission gate for shell, workspace, transport, and future external hands.
- [SCOPE-03] Model structured escalation requests, additional-permission grants, and bounded reuse rules.
- [SCOPE-04] Emit execution-governance events and UI states that explain policy decisions.
- [SCOPE-05] Update architecture/configuration docs and tests around the new policy model.

### Out of Scope

- [SCOPE-06] Hosted remote sandboxes, enterprise policy servers, or VPC-only execution infrastructure.
- [SCOPE-07] OS-specific virtualization layers beyond the abstractions required to model sandbox posture.
- [SCOPE-08] External capability connectors themselves; those belong to the external capability fabric mission.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | The runtime must resolve an explicit sandbox mode and approval policy before executing side-effecting hands in a turn. | GOAL-01 | must | Operators need a declared safety posture rather than implicit local trust. |
| FR-02 | Shell, workspace-edit, transport, and future external hands must declare the permissions they need and execute through one shared approval gate. | GOAL-01, GOAL-02 | must | Governance should be structural, not hand-specific prompt convention. |
| FR-03 | When policy blocks execution, the runtime must return a structured deny or escalation request outcome instead of silently retrying with broader authority. | GOAL-02, GOAL-03 | must | Fail-closed behavior is the core safety property of the governance model. |
| FR-04 | Escalation requests must be able to scope additional permissions or bounded reuse patterns without permanently widening all future execution. | GOAL-02 | should | Real workflows need limited exceptions without discarding the entire governance model. |
| FR-05 | Governance decisions must be emitted as transcript/trace/runtime events with enough detail to explain allow, deny, and escalation outcomes. | GOAL-03 | must | Observability is required for trust and replay. |
| FR-06 | Capability or profile downgrade must honestly disable unsupported governance features instead of pretending the active hand is protected. | GOAL-01, GOAL-03 | should | Honest degradation preserves operator understanding. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Keep the governance layer local-first and composable with existing hand contracts and harness profiles. | GOAL-01, GOAL-02 | must | The safety model should strengthen the current runtime rather than replace it. |
| NFR-02 | Blocked or degraded governance paths must fail closed with explicit diagnostics. | GOAL-02, GOAL-03 | must | Ambiguous fallback would recreate the current trust gap. |
| NFR-03 | Governance events must be replayable and human-comprehensible across TUI, web, and API projections. | GOAL-03 | must | Operators need the same explanation regardless of surface. |
| NFR-04 | Secret material and credential reachability must not expand as a side effect of introducing policy metadata or escalation flows. | GOAL-02 | must | Hardening cannot quietly widen the attack surface. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Sandbox and approval contracts | Unit and integration tests over hand-policy evaluation and fail-closed behavior | Story-level command logs and test output |
| Escalation semantics | Runtime tests and transcript proofs for deny/approve/retry flows | Story-level verification artifacts and transcript captures |
| Operator visibility | TUI/web/API projection checks for governance events | Story-level UI proofs and updated docs |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| One governance vocabulary can cover shell, workspace, transport, and future external hands. | The policy layer may need multiple hand families instead of one gate. | Validate the shared gate against the current hand adapters during decomposition. |
| Explicit execution posture will improve trust without making common local workflows unusably noisy. | The harness may need better defaults or richer escalation reuse rules. | Start with bounded policies and collect transcript evidence before broader rollout. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which sandbox postures are worth standardizing first: read-only, workspace-write, and full local access, or a finer-grained set? | Epic owner | Open |
| How should approval reuse be bounded so repeated safe commands do not create operator fatigue without becoming a hidden global bypass? | Epic owner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] The runtime declares an explicit sandbox mode and approval policy before side-effecting work begins.
- [ ] Shell and other execution hands route through one shared governance gate with fail-closed behavior.
- [ ] Escalation, denial, and approval outcomes are visible in traces and operator-facing surfaces.
- [ ] Docs and tests explain the security posture honestly enough that the default execution model is auditable.
<!-- END SUCCESS_CRITERIA -->
