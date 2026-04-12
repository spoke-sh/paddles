# VOYAGE REPORT: Govern Local Hands With Explicit Execution Policy

## Voyage Metadata
- **ID:** VGb2gViJ2
- **Epic:** VGb1c0pAN
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Execution Governance Contracts
- **ID:** VGb2k7SBM
- **Status:** done

#### Summary
Define the domain and runtime contracts for execution governance so Paddles can
reason explicitly about sandbox posture, approval policy, permission
requirements, escalation outcomes, and fail-closed degradation before the first
governed hand is wired through the new gate.

#### Acceptance Criteria
- [x] The runtime defines explicit contracts for sandbox mode, approval policy, permission requirements, escalation requests, and execution-governance outcomes. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] The contract vocabulary is hand-agnostic enough to cover shell, workspace, and future execution surfaces without provider-specific branching. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGb2k7SBM/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGb2k7SBM/EVIDENCE/ac-2.log)

### Route Shell And Workspace Hands Through The Permission Gate
- **ID:** VGb2k7kBT
- **Status:** done

#### Summary
Wrap the existing shell and workspace-edit hands in the shared permission gate
so side effects occur only when the active execution-governance profile allows
them, and blocked requests return structured deny or escalation results instead
of falling through to raw execution.

#### Acceptance Criteria
- [x] Shell and workspace-edit requests declare the permissions they need and run through one shared gate before side effects occur. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Requests that exceed the active posture return structured deny or escalation outcomes instead of retrying with broader authority. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Escalation outcomes can scope bounded reuse metadata without permanently widening later execution authority. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] Policy-evaluation failures fail closed and surface explicit diagnostics rather than implicitly widening execution authority. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGb2k7kBT/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGb2k7kBT/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGb2k7kBT/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VGb2k7kBT/EVIDENCE/ac-4.log)

### Project Execution Governance Into Trace And UI Surfaces
- **ID:** VGb2k87BR
- **Status:** done

#### Summary
Project the active execution-governance posture and per-action allow, deny, and
escalation outcomes into trace and operator-facing surfaces so the new safety
model is visible, replayable, and understandable instead of hidden in controller
logic.

#### Acceptance Criteria
- [x] Governance posture and per-action outcomes emit typed runtime or trace artifacts that downstream surfaces can consume. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] Operator-facing projections can distinguish allowed, denied, and escalated actions and explain the rationale at a high level. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] The design documents how unsupported governance features or downgraded profiles are surfaced honestly to operators. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [x] The resulting governance model remains replayable and legible across transcript and API projections. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGb2k87BR/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGb2k87BR/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGb2k87BR/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VGb2k87BR/EVIDENCE/ac-4.log)


