# VOYAGE REPORT: Planner Lane Schema Adoption

## Voyage Metadata
- **ID:** VJXf4hlYW
- **Epic:** VJXeteRQ5
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 2/2 stories complete

## Implementation Narrative
### Migrate Planner Prompts To Shared Schema
- **ID:** VJXfKtukt
- **Status:** done

#### Summary
Migrate Sift/local and HTTP/remote planner prompt builders so action-schema
text comes from the shared renderer. Provider-specific transport instructions
remain adapter-local.

#### Acceptance Criteria
- [x] Sift initial, recursive, retry, and redecision prompts consume the shared schema renderer. [SRS-01/AC-01] <!-- verify: cargo test sift_planner_prompts_use_shared_action_schema_renderer --lib, SRS-01:start:end, proof: ac-1.log-->
- [x] HTTP planner prompts consume the shared schema renderer while preserving native-tool, structured JSON, and prompt-envelope transport instructions. [SRS-02/AC-02] <!-- verify: cargo test planner_system_prompt_demands_complete_json_action_envelopes --lib, SRS-02:start:end, proof: ac-2.log-->
- [x] `rg` or equivalent proof shows no remaining adapter-local planner action JSON example lists outside the shared renderer. [SRS-03/AC-03] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "Allowed actions:|## Action Schema|Available actions:" src/infrastructure/adapters; then exit 1; else exit 0; fi', SRS-03:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJXfKtukt/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJXfKtukt/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJXfKtukt/EVIDENCE/ac-3.log)

### Prove Planner Lane Schema Parity
- **ID:** VJXfKuNks
- **Status:** done

#### Summary
Add mocked-turn tests that extract the canonical schema block from Sift and
HTTP planner prompts and compare the blocks exactly.

#### Acceptance Criteria
- [x] Mocked Sift and HTTP initial planner turns receive the same canonical schema block. [SRS-04/AC-01] <!-- verify: cargo test mocked_initial_planner_lanes_receive_same_canonical_schema_block --lib, SRS-04:start:end, proof: ac-1.log-->
- [x] Mocked Sift and HTTP recursive planner turns receive the same canonical schema block. [SRS-05/AC-02] <!-- verify: cargo test mocked_recursive_planner_lanes_receive_same_canonical_schema_block --lib, SRS-05:start:end, proof: ac-2.log-->
- [x] Test failures identify the drifting lane and prompt variant. [SRS-NFR-02/AC-03] <!-- verify: cargo test schema_block_assertions_name_drifting_lane_and_prompt_variant --lib, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJXfKuNks/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJXfKuNks/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJXfKuNks/EVIDENCE/ac-3.log)


