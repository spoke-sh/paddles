# VOYAGE REPORT: Collapse Runtime Lane Terminology

## Voyage Metadata
- **ID:** VJZ8ERr2f
- **Epic:** VJZ034dF2
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Rename Internal Lane Types To Turn Runtime Concepts
- **ID:** VJZ8OmnUP
- **Status:** done

#### Summary
Rename internal Rust runtime lane types to turn runtime concepts. This should
change active architecture names, not just user-visible labels.

#### Acceptance Criteria
- [x] Internal types such as `RuntimeLaneConfig`, `PreparedRuntimeLanes`, `PreparedModelLane`, and `PreparedGathererLane` are replaced with turn-runtime/model-client/retrieval concepts or documented migration shims. [SRS-01/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "RuntimeLaneConfig|PreparedRuntimeLanes|PreparedModelLane|PreparedGathererLane|RuntimeLaneRole|prepare_runtime_lanes|prepared_runtime_lanes|default_response_lane|default_response_role|build_lane|from_runtime_lanes" src', SRS-01:start:end, proof: ac-1.log-->
- [x] Tests and module names use the new turn-runtime vocabulary where they describe active runtime architecture. [SRS-01/AC-02] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "runtime_lane_config|remote_http_lane|prepared_turn_runtime_keep_synthesizer|prepare runtime lanes|Runtime lanes now target|Activating runtime lanes|requested runtime lanes|runtime_lanes|prepared_lanes|runtime_lane_summary" src/application/mod.rs src/application/harness_capability_posture.rs src/infrastructure/adapters/http_provider.rs src/infrastructure/cli/interactive_tui.rs src/main.rs src/infrastructure/web/mod.rs', SRS-01:start:end, proof: ac-2.log-->
- [x] Behavior stays covered by existing runtime construction and turn-loop tests after the rename. [SRS-NFR-01/AC-03] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo nextest run prepare_turn_runtime --lib && cargo test --lib turn_runtime && cargo check --all-targets', SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8OmnUP/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8OmnUP/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8OmnUP/EVIDENCE/ac-3.log)

### Rename Planner Synthesizer Gatherer Ports To Turn Phases
- **ID:** VJZ8PstqT
- **Status:** done

#### Summary
Rename planner, synthesizer, and gatherer ports/modules where they encode the
old lane architecture. Preserve behavior under clearer turn phase names such as
action selection, final rendering, retrieval, and evidence.

#### Acceptance Criteria
- [x] Internal planner/synthesizer/gatherer names are replaced where they describe lane architecture rather than unavoidable compatibility. [SRS-02/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "RecursivePlanner|SynthesizerEngine|SynthesisHandoff|ContextGatherer|GathererCapability|SynthesizerFactory|PlannerFactory|GathererFactory|HttpPlannerAdapter|build_synthesizer_engine|build_planner_engine|synthesizer_engine|planner_engine|context_gathering|sift_context_gatherer|sift_direct_gatherer|sift_autonomous_gatherer|context1_gatherer|SiftContextGathererAdapter|SiftDirectGathererAdapter|SiftAutonomousGathererAdapter|Context1GathererAdapter" src && ! rg -n "synthesizer lane|planner lane|gatherer lane|context-gathering lane|synthesizer engine|context-gathering subagents|no retrieval_provider|retrieval_provider requests|retrieval_provider backend|retrieval_provider is configured|Checked retrieval_provider" src/domain/ports src/application src/infrastructure/adapters src/infrastructure/runtime_presentation.rs src/main.rs --glob "!http_provider.rs"', SRS-02:start:end, proof: ac-1.log-->
- [x] The turn loop still exposes tested behavior for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. [SRS-02/AC-02] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --lib unified_loop && cargo test --lib execute_planner_gather_step && cargo nextest run action_selection_client_builds_from_http_provider_configuration final_rendering_client_builds_from_http_provider_configuration direct_gatherer_returns_direct_retrieval_metadata_and_evidence --no-tests pass', SRS-02:start:end, proof: ac-2.log-->
- [x] Prompt and execution-contract tests continue to expose live capabilities and enforced constraints without synthetic controller-authored plans. [SRS-02/AC-03] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --lib execution_contract && cargo test --lib runtime_posture_projection && cargo test agent_loop_prompt --all-targets', SRS-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8PstqT/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8PstqT/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8PstqT/EVIDENCE/ac-3.log)

### Retire Runtime Lane Language From Public Surfaces
- **ID:** VJZ8QTY4i
- **Status:** done

#### Summary
Retire runtime lane language from public surfaces after the internal rename is
complete. Any remaining old terms must be explicit compatibility aliases or
historical artifacts, not active product vocabulary.

#### Acceptance Criteria
- [x] CLI help, TUI/web route copy, docs, and prompt prose no longer present planner, synthesizer, or gatherer as runtime lanes. [SRS-03/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --bin paddles cli_help_presents_turn_phase_flags_and_hides_legacy_lane_aliases && ! rg -n -i "planner lane|synthesizer lane|gatherer lane|runtime lanes|runtime lane|context-gathering lane|recursive planner lane" README.md ARCHITECTURE.md CONFIGURATION.md POLICY.md INSTRUCTIONS.md apps/docs src', SRS-03:start:end, proof: ac-1.log-->
- [x] String scans or targeted tests cover old public phrases such as "planner lane", "synthesizer lane", "gatherer lane", and "runtime lanes". [SRS-03/AC-02] <!-- verify: cargo test --all-targets agent_loop_prompt_vocabulary, SRS-03:start:end, proof: ac-2.log-->
- [x] Retained legacy aliases are documented as migration shims and point to action-selection, final-rendering, or retrieval terminology. [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end, proof: ac-3.log-->
- [x] Tests prove turn-loop behavior remains covered for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. [SRS-05/AC-04] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --lib unified_loop && cargo test --lib execute_planner_gather_step && cargo nextest run action_selection_client_builds_from_http_provider_configuration final_rendering_client_builds_from_http_provider_configuration direct_gatherer_returns_direct_retrieval_metadata_and_evidence --no-tests pass', SRS-05:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8QTY4i/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8QTY4i/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8QTY4i/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VJZ8QTY4i/EVIDENCE/ac-4.log)


