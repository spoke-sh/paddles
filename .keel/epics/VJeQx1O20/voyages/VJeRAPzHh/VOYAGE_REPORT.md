# VOYAGE REPORT: Move Turn Contract Into Agent Loop

## Voyage Metadata
- **ID:** VJeRAPzHh
- **Epic:** VJeQx1O20
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Rename Collaboration Runtime Contract
- **ID:** VJeRX3FXi
- **Status:** done

#### Summary
Rename and reshape the agent-loop `collaboration` field into a turn contract or turn policy concept that accurately describes mutation posture, output contract, clarification policy, and mode status.

#### Acceptance Criteria
- [x] Agent-loop/application internals use `turn_contract` or `turn_policy` naming instead of `collaboration` for runtime policy. [SRS-01/AC-01] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "context\\.collaboration|collaboration_runtime_notes|CollaborationModeResult" src/application src/domain src/infrastructure; then exit 1; else test $? -eq 1; fi', SRS-01:start:end, proof: ac-1.log-->
- [x] Existing planning, execution, and review semantics are preserved under the renamed contract. [SRS-02/AC-02] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test turn_contract_preserves_mode_semantics -- --nocapture', SRS-02:start:end, proof: ac-2.log-->
- [x] Any retained collaboration terminology is documented as external or serialized compatibility. [SRS-NFR-02/AC-03] <!-- verify: sh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "collaboration" src ARCHITECTURE.md CONFIGURATION.md', SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRX3FXi/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRX3FXi/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRX3FXi/EVIDENCE/ac-3.log)

### Move Turn Obligations Into Loop State
- **ID:** VJeRYVkyL
- **Status:** done

#### Summary
Move edit, commit, review, and grounding obligations into loop state or instruction-frame data so they guide model action selection inside the loop rather than forcing a first action in `turn.rs`.

#### Acceptance Criteria
- [x] Edit and commit obligations are attached to the first loop request as instruction-frame or loop-state data. [SRS-03/AC-01] <!-- verify: cargo test turn_obligations_are_loop_inputs -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Grounding and review pressure no longer force a pre-loop bootstrap action. [SRS-04/AC-02] <!-- verify: cargo test grounding_and_review_pressure_are_loop_context -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
- [x] Read-only, review, and execution mutation behavior remains enforced after model selection and before execution. [SRS-02/AC-03] <!-- verify: cargo test turn_contract_blocks_mutation_inside_loop -- --nocapture, SRS-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJeRYVkyL/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJeRYVkyL/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJeRYVkyL/EVIDENCE/ac-3.log)


