# VOYAGE REPORT: Establish A Replayable Turn And Thread Control Substrate

## Voyage Metadata
- **ID:** VGbPWnUh2
- **Epic:** VGb1c1AAK
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Turn And Thread Control Contracts
- **ID:** VGbPaBICN
- **Status:** done

#### Summary
Define the typed turn-control, thread-control, and runtime-item contracts so
Paddles can express same-turn steering, thread lifecycle transitions, and live
plan or diff state as durable runtime semantics instead of surface-specific
prompt conventions.

#### Acceptance Criteria
- [x] The runtime defines typed contracts for turn and thread control operations, control results, and shared runtime items for plan, diff, command, file, and control-state updates. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] The contract vocabulary defines turn and thread control semantics independently of any one operator surface or prompt phrasing. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] The contract model builds directly on the existing recorder, replay, and thread-lineage substrate instead of introducing a parallel state store. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGbPaBICN/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGbPaBICN/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGbPaBICN/EVIDENCE/ac-3.log)

### Wire Same-Turn Steering Through Replayable Control Flow
- **ID:** VGbPaBmCK
- **Status:** done

#### Summary
Wire same-turn steering, interruption, and lineage-aware thread lifecycle
transitions through replayable control flow so the recursive harness can be
steered intentionally without falling back to opaque queued prompts or hidden
thread mutation.

#### Acceptance Criteria
- [x] Same-turn steering and interruption flow through replayable control records with bounded fallback when a requested action cannot apply. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end, proof: ac-1.log-->
- [x] Fork, resume, and rollback or archive style transitions preserve durable thread lineage and replayability. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Accepted steering and interruption requests stay attached to the active turn lifecycle instead of degrading back into queued follow-up prompts. [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end, proof: ac-3.log-->
- [x] The steering path reports explicit bounded fallback when a requested control action is unsafe or unsupported in the current execution window. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGbPaBmCK/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGbPaBmCK/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGbPaBmCK/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VGbPaBmCK/EVIDENCE/ac-4.log)

### Project Live Control State Across Operator Surfaces
- **ID:** VGbPaCCCI
- **Status:** done

#### Summary
Project live control state, plan updates, diff state, command summaries, and
file-change artifacts across transcript, TUI, web, and API surfaces so a
running turn becomes legible without reading raw trace internals.

#### Acceptance Criteria
- [x] Active turns emit typed runtime items for plan updates, diff updates, command summaries, file changes, and control-state transitions. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end, proof: ac-1.log-->
- [x] TUI, web, and API projections render one shared control and runtime-item vocabulary without divergent semantics. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end, proof: ac-2.log-->
- [x] Invalid or stale control requests surface explicit rejected, stale, or unavailable status instead of mutating hidden thread state. [SRS-06/AC-01] <!-- verify: manual, SRS-06:start:end, proof: ac-3.log-->
- [x] Transcript and UI surfaces keep control transitions readable and surface degraded or unsupported states honestly. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-4.log-->
- [x] Shared control-state projections remain deterministic enough for focused replay and projection proofs. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-5.log-->
- [x] The resulting control plane preserves the local-first recursive execution model while exposing live operator state. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end, proof: ac-6.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VGbPaCCCI/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VGbPaCCCI/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VGbPaCCCI/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VGbPaCCCI/EVIDENCE/ac-4.log)
- [ac-5.log](../../../../stories/VGbPaCCCI/EVIDENCE/ac-5.log)
- [ac-6.log](../../../../stories/VGbPaCCCI/EVIDENCE/ac-6.log)


