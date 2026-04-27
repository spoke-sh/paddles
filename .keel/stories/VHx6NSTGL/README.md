---
# system-managed
id: VHx6NSTGL
status: done
created_at: 2026-04-26T18:54:45
updated_at: 2026-04-26T19:01:23
# authored
title: Expose GPT-5.5 And OpenAI Pro Model IDs
type: feat
operator-signal:
scope: VHx5jpzIB/VHx6M7kqq
index: 1
started_at: 2026-04-26T18:56:22
completed_at: 2026-04-26T19:01:23
---

# Expose GPT-5.5 And OpenAI Pro Model IDs

## Summary

Expose current OpenAI GPT-5.5 and text/reasoning pro model IDs through the
provider catalog, with tests proving selectable IDs, thinking modes, and
Responses-oriented pro capability routing.

## Acceptance Criteria

- [x] The OpenAI provider catalog accepts `gpt-5.5`, `gpt-5.5-pro`, `gpt-5.4-pro`, `gpt-5.2-pro`, `gpt-5-pro`, `o3-pro`, and `o1-pro`. [SRS-01/AC-01] <!-- verify: cargo test -q openai_provider_exposes_additional_model_ids -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] GPT-5.5 thinking modes match the documented `none`, `low`, `medium`, `high`, and `xhigh` set and runtime labels preserve selected effort. [SRS-02/AC-02] <!-- verify: cargo test -q openai_gpt_5_5_models_expose_parameterized_thinking_modes -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
- [x] OpenAI pro model paths use the supported Responses-oriented capability surface and documented thinking controls. [SRS-03/AC-03] <!-- verify: cargo test -q openai_pro_models_expose_only_documented_thinking_controls -- --nocapture && cargo test -q openai_transport_supports_responses_only_pro_models -- --nocapture && cargo test -q provider_capability_matrix_covers_documented_provider_paths -- --nocapture, SRS-03:start:end, proof: ac-3.log-->
- [x] Configuration docs embed the current generated OpenAI provider capability matrix. [SRS-04/AC-04] <!-- verify: cargo test -q configuration_docs_embed_current_provider_capability_matrix -- --nocapture, SRS-04:start:end, proof: ac-4.log-->
