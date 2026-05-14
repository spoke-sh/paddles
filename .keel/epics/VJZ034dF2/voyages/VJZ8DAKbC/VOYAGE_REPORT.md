# VOYAGE REPORT: Migrate Provider Preferences To Turn Runtime Config

## Voyage Metadata
- **ID:** VJZ8DAKbC
- **Epic:** VJZ034dF2
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 4/4 stories complete

## Implementation Narrative
### Introduce Turn Runtime Preference Schema
- **ID:** VJZ8LOzJi
- **Status:** done

#### Summary
Introduce the canonical turn-runtime preference schema. New code should describe
model clients and turn phases directly instead of persisting planner,
synthesizer, gatherer, or runtime-lane settings.

#### Acceptance Criteria
- [x] Tests define the new preference shape using action-selection, final-rendering, retrieval, model-client, and turn-runtime terminology. [SRS-01/AC-01] <!-- verify: cargo nextest run turn_runtime_preferences_capture_model_clients_and_retrieval turn_runtime_preferences_record_shared_model_clients_without_lane_names, SRS-01:start:end, proof: ac-1.log-->
- [x] New preference writes do not emit planner, synthesizer, gatherer, or lane-shaped field names. [SRS-01/AC-02] <!-- verify: cargo nextest run turn_runtime_preference_store_writes_canonical_shape_without_lane_terms turn_runtime_preference_store_round_trips_preferences, SRS-01:start:end, proof: ac-2.log-->
- [x] Runtime construction consumes normalized turn-runtime preferences. [SRS-01/AC-03] <!-- verify: cargo nextest run load_layers_runtime_preferences_after_authored_config runtime_preferences_override_workspace_config_for_lane_fields openai_gpt_5_4_thinking_mode_selection_queues_runtime_update_from_prompt_box, SRS-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8LOzJi/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8LOzJi/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8LOzJi/EVIDENCE/ac-3.log)

### Migrate Legacy Runtime Lane Preferences
- **ID:** VJZ8Lz4V9
- **Status:** done

#### Summary
Keep legacy lane-shaped config readable as migration input while making the new
turn-runtime preference shape the only write target. Legacy Sift model-provider
values still hard-fail rather than remapping silently.

#### Acceptance Criteria
- [x] Migration fixture tests prove legacy runtime-lane config is read and normalized into turn-runtime preferences. [SRS-02/AC-01] <!-- verify: cargo nextest run legacy_runtime_lane_preferences_migrate_into_turn_runtime_shape, SRS-02:start:end, proof: ac-1.log-->
- [x] Persistence tests prove new writes use only the turn-runtime preference shape. [SRS-02/AC-02] <!-- verify: cargo nextest run turn_runtime_preference_store_writes_canonical_shape_without_lane_terms turn_runtime_preference_store_round_trips_preferences, SRS-02:start:end, proof: ac-2.log-->
- [x] Legacy lane config containing Sift model-provider values fails with the approved `ollama:<model>` migration hint. [SRS-NFR-01/AC-03] <!-- verify: cargo nextest run legacy_runtime_lane_preferences_reject_sift_model_provider_with_migration_hint, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8Lz4V9/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8Lz4V9/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8Lz4V9/EVIDENCE/ac-3.log)

### Document Ollama Local HTTP Defaults
- **ID:** VJZ8MXfkO
- **Status:** done

#### Summary
Update configuration and setup docs so local-first inference is documented as
an HTTP-hosted model service, with Ollama examples using `ollama:<model>`.

#### Acceptance Criteria
- [x] `CONFIGURATION.md` documents turn-runtime preference precedence and the new preference shape. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] Local inference examples use `ollama:<model>` without naming a fixed default model. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end, proof: ac-2.log-->
- [x] Docs no longer describe runtime lanes as the canonical provider preference model. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VJZ8MXfkO/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VJZ8MXfkO/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VJZ8MXfkO/EVIDENCE/ac-3.log)

### Preserve HTTP Provider Credential Rules
- **ID:** VJZ9qwaWd
- **Status:** done

#### Summary
Preserve HTTP provider credential and availability behavior while the preference
schema changes. Optional local providers such as Ollama should remain usable
without credentials, while credentialed HTTP providers fail closed when required
secrets are missing.

#### Acceptance Criteria
- [x] Tests prove optional Ollama-style local HTTP providers remain available without credentials. [SRS-04/AC-01] <!-- verify: cargo nextest run optional_local_provider_stays_enabled_without_credentials mediator_allows_optional_ollama_provider_without_credentials ollama_model_clients_build_without_credentials, SRS-04:start:end -->
- [x] Tests prove required HTTP providers fail closed with provider-specific credential guidance when secrets are missing. [SRS-04/AC-02] <!-- verify: cargo nextest run required_remote_provider_is_disabled_when_missing_credentials mediator_fails_closed_for_missing_required_provider_credentials, SRS-04:start:end -->
- [x] Preference migration does not bypass existing credential-store or transport-mediator boundaries. [SRS-04/AC-03] <!-- verify: cargo nextest run legacy_runtime_lane_preferences_migrate_into_turn_runtime_shape migrated_turn_runtime_preferences_do_not_bypass_transport_mediator_credentials, SRS-04:start:end -->


