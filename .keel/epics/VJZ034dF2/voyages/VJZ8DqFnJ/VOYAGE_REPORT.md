# VOYAGE REPORT: Remove In-Process Sift Inference Code

## Voyage Metadata
- **ID:** VJZ8DqFnJ
- **Epic:** VJZ034dF2
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 3/3 stories complete

## Implementation Narrative
### Delete Sift Agent And Planner Inference Adapters
- **ID:** VJZ8N7avc
- **Status:** done

#### Summary
Delete the Sift action-selection and final-rendering inference adapters after
HTTP-only runtime construction is proven. Any remaining Sift code must be
retrieval-specific or compatibility parsing that fails before runtime.

#### Acceptance Criteria
- [x] Compile failures or targeted tests first identify every remaining active reference to Sift inference adapters. [SRS-01/AC-01] <!-- verify: sh -lc 'test ! -e src/infrastructure/adapters/sift_agent.rs && test ! -e src/infrastructure/adapters/sift_planner.rs && ! rg -n "sift_agent|SiftAgentAdapter|SiftPlannerAdapter|pub mod sift_planner|pub mod sift_agent" src tests', SRS-01:start:end -->
- [x] Sift action-selection and final-rendering inference adapters are deleted or made unreachable from runtime construction. [SRS-01/AC-02] <!-- verify: cargo clippy --all-targets -- -D warnings, SRS-01:start:end -->
- [x] Legacy Sift model-provider inputs still fail with the approved migration hint rather than panicking or falling through. [SRS-NFR-02/AC-03] <!-- verify: cargo nextest run action_selection_client_rejects_legacy_sift_provider_with_migration_hint final_rendering_client_rejects_legacy_sift_provider_with_migration_hint prepare_runtime_lanes_rejects_legacy_sift_synthesizer_before_construction prepare_runtime_lanes_rejects_legacy_sift_planner_before_construction sift_direct_boundary_can_be_prepared_without_local_model_paths direct_gatherer_returns_direct_retrieval_metadata_and_evidence, SRS-NFR-02:start:end -->

### Remove Inference-Only Sift Model Dependencies
- **ID:** VJZ8NgA6d
- **Status:** done

#### Summary
Remove inference-only dependencies and build surfaces that become unused after
the Sift inference adapters are gone. Retrieval dependencies should remain only
when retrieval code still needs them.

#### Acceptance Criteria
- [x] Dependency review identifies Candle/Qwen/tokenizer or other inference-only crates that no remaining active code uses. [SRS-02/AC-01] <!-- verify: sh -lc 'if cargo tree --depth 1 -e normal | rg "candle|tokenizers|hf-hub"; then exit 1; fi; ! rg -n "candle_core|candle_nn|candle_transformers|tokenizers::|hf_hub::|HFHubAdapter|SiftRegistryAdapter|sift_registry" src Cargo.toml', SRS-02:start:end -->
- [x] Cargo/build configuration removes inference-only dependencies that are no longer needed. [SRS-02/AC-02] <!-- verify: cargo clippy --all-targets -- -D warnings, SRS-02:start:end -->
- [x] HTTP provider and Sift retrieval tests remain green after dependency cleanup. [SRS-NFR-01/AC-03] <!-- verify: cargo nextest run prepare_runtime_lanes_preserves_local_gatherer_without_model_loading prepare_runtime_lanes_preserves_sift_direct_retrieval_with_http_inference direct_gatherer_returns_direct_retrieval_metadata_and_evidence action_selection_client_builds_from_http_provider_configuration final_rendering_client_builds_from_http_provider_configuration provider_capability_matrix_covers_documented_provider_paths configuration_docs_embed_current_provider_capability_matrix known_state_space_models_include_remote_catalog_entries_without_legacy_sift_models, SRS-NFR-01:start:end -->

### Purge Local Model Loading Documentation
- **ID:** VJZ8OERIu
- **Status:** done

#### Summary
Purge documentation that teaches paddles-owned local inference model loading.
Docs should direct local-first users to HTTP-hosted model services and the
`ollama:<model>` provider form.

#### Acceptance Criteria
- [x] README, ARCHITECTURE, CONFIGURATION, POLICY, and build notes no longer describe paddles-owned local inference model loading as supported behavior. [SRS-03/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n -i "local inference|local models|model loading|download.*model|prepare local model|load local model|hf_token|Hugging Face|HF Hub|Candle|qwen-1.5b|qwen3.5|qwen-coder|--model qwen|sift.*inference|in-process.*inference|sift_agent|sift_planner" README.md ARCHITECTURE.md CONFIGURATION.md POLICY.md INSTRUCTIONS.md apps/docs/docs apps/docs/src apps/docs/docusaurus.config.ts apps/docs/sidebars.ts apps/docs/README.md justfile package.json Cargo.toml', SRS-03:start:end -->
- [x] Local setup docs point to HTTP-hosted local model services and `ollama:<model>`. [SRS-03/AC-02] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "ollama:<model>|ollama:qwen3" README.md CONFIGURATION.md apps/docs/docs/start-here/first-turn.mdx apps/docs/docs/concepts/model-routing.mdx && rg -n "local HTTP model service|Local HTTP model client|model process is outside Paddles" README.md CONFIGURATION.md apps/docs/docs/start-here/first-turn.mdx apps/docs/docs/concepts/model-routing.mdx apps/docs/src/pages/index.tsx', SRS-03:start:end -->
- [x] Sift retrieval/indexing documentation, if still present, is clearly separated from model inference. [SRS-04/AC-03] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "Sift remains.*retrieval|sift-direct.*retrieval|sift.*retrieves|retrieval/indexing" README.md ARCHITECTURE.md CONFIGURATION.md apps/docs/docs/concepts/search-retrieval.mdx apps/docs/docs/concepts/model-routing.mdx && ! rg -n -i "sift.*inference|sift_agent|sift_planner|qwen-1.5b" README.md ARCHITECTURE.md CONFIGURATION.md apps/docs/docs apps/docs/src', SRS-04:start:end -->


