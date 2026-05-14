---
# system-managed
id: VJZ8NgA6d
status: done
created_at: 2026-05-13T21:29:56
updated_at: 2026-05-13T22:41:07
# authored
title: Remove Inference-Only Sift Model Dependencies
type: chore
operator-signal:
scope: VJZ034dF2/VJZ8DqFnJ
index: 2
started_at: 2026-05-13T22:33:51
completed_at: 2026-05-13T22:41:07
---

# Remove Inference-Only Sift Model Dependencies

## Summary

Remove inference-only dependencies and build surfaces that become unused after
the Sift inference adapters are gone. Retrieval dependencies should remain only
when retrieval code still needs them.

## Acceptance Criteria

- [x] Dependency review identifies Candle/Qwen/tokenizer or other inference-only crates that no remaining active code uses. [SRS-02/AC-01] <!-- verify: sh -lc 'if cargo tree --depth 1 -e normal | rg "candle|tokenizers|hf-hub"; then exit 1; fi; ! rg -n "candle_core|candle_nn|candle_transformers|tokenizers::|hf_hub::|HFHubAdapter|SiftRegistryAdapter|sift_registry" src Cargo.toml', SRS-02:start:end -->
- [x] Cargo/build configuration removes inference-only dependencies that are no longer needed. [SRS-02/AC-02] <!-- verify: cargo clippy --all-targets -- -D warnings, SRS-02:start:end -->
- [x] HTTP provider and Sift retrieval tests remain green after dependency cleanup. [SRS-NFR-01/AC-03] <!-- verify: cargo nextest run prepare_runtime_lanes_preserves_local_gatherer_without_model_loading prepare_runtime_lanes_preserves_sift_direct_retrieval_with_http_inference direct_gatherer_returns_direct_retrieval_metadata_and_evidence action_selection_client_builds_from_http_provider_configuration final_rendering_client_builds_from_http_provider_configuration provider_capability_matrix_covers_documented_provider_paths configuration_docs_embed_current_provider_capability_matrix known_state_space_models_include_remote_catalog_entries_without_legacy_sift_models, SRS-NFR-01:start:end -->
