---
# system-managed
id: VHXLSDBjB
status: done
created_at: 2026-04-22T09:11:33
updated_at: 2026-04-22T10:30:49
# authored
title: Expose Thinking Modes Across Supported Providers
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipyBc
index: 4
started_at: 2026-04-22T10:15:43
completed_at: 2026-04-22T10:30:49
---

# Expose Thinking Modes Across Supported Providers

## Summary

Expose supported thinking modes, reasoning effort controls, or explicit `none`
results across every supported provider/model path so runtime selection and
configuration no longer depend on provider-specific hard-coding.

## Acceptance Criteria

- [x] Every supported provider/model path exposes supported thinking modes or an explicit `none`/unsupported result through provider catalogs and runtime configuration. [SRS-07/AC-01] <!-- verify: cargo test provider_catalog_exposes_thinking_modes_across_supported_providers -- --nocapture && cargo test runtime_model_ids_preserve_provider_specific_thinking_modes_when_needed -- --nocapture && cargo test inception_thinking_mode_maps_to_reasoning_effort_and_summary -- --nocapture && cargo test anthropic_none_thinking_mode_omits_extended_thinking_payload -- --nocapture && cargo test gemini_high_thinking_mode_maps_to_budgeted_thinking_config -- --nocapture && cargo test moonshot_k2_6_none_thinking_mode_maps_to_disabled_toggle -- --nocapture && cargo test ollama_thinking_modes_map_to_boolean_and_level_controls -- --nocapture && cargo test explicit_none_only_thinking_catalogs_do_not_block_model_selection -- --nocapture, SRS-07:start:end -->
- [x] Thinking-mode catalogs stay synchronized with the actual provider capability surface and fallback behavior. [SRS-NFR-03/AC-02] <!-- verify: cargo test configuration_docs_embed_current_provider_capability_matrix -- --nocapture && cargo test capability_surface_ -- --nocapture && npm --workspace @paddles/docs run build, SRS-NFR-03:start:end -->
