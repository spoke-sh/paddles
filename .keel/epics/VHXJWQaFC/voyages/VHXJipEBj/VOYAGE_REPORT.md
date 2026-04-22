# VOYAGE REPORT: Deliberation Substrate And Continuation Contracts

## Voyage Metadata
- **ID:** VHXJipEBj
- **Epic:** VHXJWQaFC
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Define Deliberation Capability Surface And Adapter State
- **ID:** VHXK90tXf
- **Status:** done

#### Summary
Define the provider-agnostic deliberation capability surface and the opaque
adapter state shape that later provider implementations will plug into, without
changing canonical transcript/render state or replacing paddles `rationale`.

#### Acceptance Criteria
- [x] Provider/model negotiation classifies deliberation support explicitly for every provider path needed by the runtime. [SRS-01/AC-01] <!-- verify: cargo test capability_surface_ -- --nocapture, SRS-01:start:end -->
- [x] Adapter turn interfaces can return and accept opaque deliberation state separately from authored response and paddles rationale. [SRS-02/AC-02] <!-- verify: cargo test provider_turn_request_and_response_keep_deliberation_state_separate_from_content -- --nocapture, SRS-02:start:end -->

### Record Debug-Scoped Deliberation Artifacts Without Polluting Rationale
- **ID:** VHXK91IXg
- **Status:** done

#### Summary
Add the bounded debug or forensic recording path for provider-native reasoning
artifacts so maintainers can inspect continuity behavior without contaminating
canonical turn records or paddles-authored rationale.

#### Acceptance Criteria
- [x] Provider-native reasoning artifacts, if recorded, live on a debug-scoped path separate from canonical transcript/render persistence. [SRS-04/AC-01] <!-- verify: cargo test moonshot_reasoning_artifacts_record_on_forensic_debug_path_only -- --nocapture, SRS-04:start:end -->
- [x] Contract tests cover one native-continuation provider and one explicit no-op provider. [SRS-05/AC-02] <!-- verify: cargo test openai_toggle_only_models_do_not_emit_deliberation_artifacts -- --nocapture, SRS-05:start:end -->

### Preserve Moonshot Reasoning Continuity Across Tool Turns
- **ID:** VHXK91nXh
- **Status:** done

#### Summary
Use the new substrate to carry Moonshot/Kimi reasoning continuity through
recursive tool/result turns so the provider can preserve its native thinking
state without leaking raw reasoning into canonical turn output.

#### Acceptance Criteria
- [x] Moonshot/Kimi preserves required provider-native reasoning continuity across tool/result turns. [SRS-03/AC-01] <!-- verify: cargo test moonshot_prompt_envelope_replays_reasoning_state_before_tool_results -- --nocapture, SRS-03:start:end -->
- [x] Canonical transcript/render output remains free of raw Moonshot reasoning artifacts. [SRS-04/AC-02] <!-- verify: cargo test moonshot_prompt_envelope_captures_reasoning_tool_state_without_exposing_it_as_content -- --nocapture, SRS-04:start:end -->


