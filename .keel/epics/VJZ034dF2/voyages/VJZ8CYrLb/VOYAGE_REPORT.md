# VOYAGE REPORT: Route Runtime Inference Through HTTP Model Clients

## Voyage Metadata
- **ID:** VJZ8CYrLb
- **Epic:** VJZ034dF2
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Resolve Action Selection Through HTTP Model Clients
- **ID:** VJZ8JjSkG
- **Status:** done

#### Summary
Move action-selection model construction to the HTTP model-client boundary.
This story removes Sift model-path preparation from the action-selection path
while preserving provider capability negotiation.

#### Acceptance Criteria
- [x] A failing test is added first proving action-selection runtime construction never receives local `ModelPaths`. [SRS-01/AC-01] <!-- verify: cargo nextest run action_selection_http_client_rejects_local_model_paths prepare_runtime_lanes_treats_inception_as_remote_http_lane_without_local_paths, SRS-01:start:end, proof: ac-1.log-->
- [x] Action-selection clients are built through HTTP provider configuration and capability negotiation. [SRS-01/AC-02] <!-- verify: cargo nextest run action_selection_client_builds_from_http_provider_configuration, SRS-01:start:end, proof: ac-2.log-->
- [x] Legacy Sift action-selection provider config fails with the approved `ollama:<model>` migration hint. [SRS-01/AC-03] <!-- verify: cargo nextest run action_selection_client_rejects_legacy_sift_provider_with_migration_hint prepare_runtime_lanes_rejects_legacy_sift_planner_before_construction, SRS-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8JjSkG/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8JjSkG/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8JjSkG/EVIDENCE/ac-3.log)

### Resolve Final Rendering Through HTTP Model Clients
- **ID:** VJZ8KHZvT
- **Status:** done

#### Summary
Move final-rendering model construction to the HTTP model-client boundary. The
turn loop should receive a final-rendering client without paddles preparing a
local inference model.

#### Acceptance Criteria
- [x] A failing test is added first proving final-rendering runtime construction never receives local `ModelPaths`. [SRS-02/AC-01] <!-- verify: cargo nextest run final_rendering_http_client_rejects_local_model_paths prepare_runtime_lanes_treats_inception_as_remote_http_lane_without_local_paths, SRS-02:start:end, proof: ac-1.log-->
- [x] Final-rendering clients are built through HTTP provider configuration and capability negotiation. [SRS-02/AC-02] <!-- verify: cargo nextest run final_rendering_client_builds_from_http_provider_configuration final_rendering_client_rejects_legacy_sift_provider_with_migration_hint, SRS-02:start:end, proof: ac-2.log-->
- [x] HTTP provider tests for structured final answers, retries, and provider-specific schema behavior remain green. [SRS-02/AC-03] <!-- verify: cargo nextest run openai_provider_normalizes_structured_final_answers send_with_retry_retries_on_429_then_succeeds gemini_provider_executes_a_full_turn_against_a_mock_server anthropic_provider_executes_a_full_turn_against_a_mock_server moonshot_provider_executes_full_turn_without_planner_tool_choice, SRS-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8KHZvT/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8KHZvT/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8KHZvT/EVIDENCE/ac-3.log)

### Preserve Sift Retrieval Outside Inference Cleanup
- **ID:** VJZ8KqF7U
- **Status:** done

#### Summary
Protect Sift retrieval/indexing from the inference cleanup. The story should
prove retrieval remains separately selectable and does not depend on removed
model-provider behavior.

#### Acceptance Criteria
- [x] Tests prove legacy Sift model-provider branches fail before runtime construction using the ADR compatibility policy. [SRS-03/AC-01] <!-- verify: cargo nextest run prepare_runtime_lanes_rejects_legacy_sift_synthesizer_before_construction prepare_runtime_lanes_rejects_legacy_sift_planner_before_construction, SRS-03:start:end, proof: ac-1.log-->
- [x] Tests prove Sift retrieval/indexing can be prepared without Sift model-provider inference paths. [SRS-04/AC-02] <!-- verify: cargo nextest run prepare_runtime_lanes_preserves_sift_direct_retrieval_with_http_inference sift_direct_boundary_can_be_prepared_without_local_model_paths, SRS-04:start:end, proof: ac-2.log-->
- [x] Retrieval provider selection remains independent from action-selection and final-rendering model-client selection. [SRS-04/AC-03] <!-- verify: cargo nextest run prepare_runtime_lanes_preserves_sift_direct_retrieval_with_http_inference prepare_runtime_lanes_resolves_local_gatherer_paths_independent_of_http_inference, SRS-04:start:end, proof: ac-3.log-->
- [x] Any inference cleanup that would require deleting retrieval/indexing is stopped and split into a later mission decision. [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end, proof: ac-4.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8KqF7U/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8KqF7U/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8KqF7U/EVIDENCE/ac-3.log)
- [ac-4.log](../../../../stories/VJZ8KqF7U/EVIDENCE/ac-4.log)


