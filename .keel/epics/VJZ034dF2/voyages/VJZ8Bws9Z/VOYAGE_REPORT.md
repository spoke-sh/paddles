# VOYAGE REPORT: Adopt HTTP-Only Inference Decision

## Voyage Metadata
- **ID:** VJZ8Bws9Z
- **Epic:** VJZ034dF2
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Adopt HTTP-Only Model Inference ADR
- **ID:** VJZ8IcONz
- **Status:** done

#### Summary
Adopt the ADR that makes HTTP model clients the only supported inference
boundary for action selection and final rendering. The story should also add
guardrails that keep docs and future code aligned with the decision.

#### Acceptance Criteria
- [x] ADR states paddles no longer loads inference models in-process for action selection or final rendering. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] ADR states local-first inference is supported through HTTP model services and uses `ollama:<model>` as the canonical local provider form. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] Architecture/configuration docs reference the ADR and stop presenting in-process local model loading as the future inference path. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8IcONz/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8IcONz/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8IcONz/EVIDENCE/ac-3.log)

### Codify Legacy Sift Provider Migration Failure
- **ID:** VJZ8JB2ZB
- **Status:** done

#### Summary
Codify the compatibility behavior for old Sift model-provider settings. Legacy
Sift inference config must fail explicitly with an actionable migration hint
instead of silently changing providers.

#### Acceptance Criteria
- [x] Tests prove `provider = "sift"` and equivalent planner/final-rendering legacy provider selections fail before runtime construction. [SRS-02/AC-01] <!-- verify: cargo nextest run provider_config_rejects_legacy_sift_model_provider planner_provider_config_rejects_legacy_sift_model_provider prepare_runtime_lanes_rejects_legacy_sift, SRS-02:start:end, proof: ac-1.log-->
- [x] The failure message states that `sift` no longer performs model inference and tells the operator to choose an HTTP provider such as `ollama:<model>`. [SRS-02/AC-02] <!-- verify: cargo nextest run provider_config_rejects_legacy_sift_model_provider planner_provider_config_rejects_legacy_sift_model_provider prepare_runtime_lanes_rejects_legacy_sift, SRS-02:start:end, proof: ac-2.log-->
- [x] Sift retrieval/indexing selections are not rejected by this model-provider compatibility policy. [SRS-02/AC-03] <!-- verify: cargo nextest run sift_direct_boundary_can_be_prepared_without_local_model_paths, SRS-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8JB2ZB/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8JB2ZB/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8JB2ZB/EVIDENCE/ac-3.log)

### Document HTTP-Only Inference Decision
- **ID:** VJZ9qJ9G6
- **Status:** done

#### Summary
Update the owning architecture and configuration docs after the ADR is adopted.
This story makes the decision visible to operators without changing runtime
behavior beyond the documented compatibility policy.

#### Acceptance Criteria
- [x] Architecture and configuration docs point to the HTTP-only inference ADR. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Docs stop presenting in-process model loading as the future-supported inference path. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Documentation uses `ollama:<model>` as the local HTTP inference form without naming a fixed default model. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ9qJ9G6/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ9qJ9G6/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ9qJ9G6/EVIDENCE/ac-3.log)


