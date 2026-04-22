---
# system-managed
id: VHXK942dC
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T10:10:21
# authored
title: Publish Provider Capability Matrix Tests And Operator Docs
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipyBc
index: 3
started_at: 2026-04-22T10:05:49
completed_at: 2026-04-22T10:10:21
---

# Publish Provider Capability Matrix Tests And Operator Docs

## Summary

Ship the cross-provider capability matrix, contract tests, and operator-facing
configuration guidance so reasoning behavior is explicit for every supported
provider and stays synchronized with implementation.

## Acceptance Criteria

- [x] The repository publishes a provider capability matrix and contract tests for Moonshot, OpenAI, Anthropic, Gemini, Inception, Ollama, and Sift. [SRS-06/AC-01] <!-- verify: cargo test provider_capability_matrix_covers_documented_provider_paths -- --nocapture && cargo test capability_surface_ -- --nocapture, SRS-06:start:end -->
- [x] Operator/configuration docs stay synchronized with the actual capability surface. [SRS-NFR-03/AC-02] <!-- verify: cargo test configuration_docs_embed_current_provider_capability_matrix -- --nocapture && npm --workspace @paddles/docs run build, SRS-NFR-03:start:end -->
