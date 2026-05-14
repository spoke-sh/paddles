---
# system-managed
id: VJZ8OERIu
status: done
created_at: 2026-05-13T21:29:58
updated_at: 2026-05-13T22:47:28
# authored
title: Purge Local Model Loading Documentation
type: docs
operator-signal:
scope: VJZ034dF2/VJZ8DqFnJ
index: 3
started_at: 2026-05-13T22:42:26
completed_at: 2026-05-13T22:47:28
---

# Purge Local Model Loading Documentation

## Summary

Purge documentation that teaches paddles-owned local inference model loading.
Docs should direct local-first users to HTTP-hosted model services and the
`ollama:<model>` provider form.

## Acceptance Criteria

- [x] README, ARCHITECTURE, CONFIGURATION, POLICY, and build notes no longer describe paddles-owned local inference model loading as supported behavior. [SRS-03/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n -i "local inference|local models|model loading|download.*model|prepare local model|load local model|hf_token|Hugging Face|HF Hub|Candle|qwen-1.5b|qwen3.5|qwen-coder|--model qwen|sift.*inference|in-process.*inference|sift_agent|sift_planner" README.md ARCHITECTURE.md CONFIGURATION.md POLICY.md INSTRUCTIONS.md apps/docs/docs apps/docs/src apps/docs/docusaurus.config.ts apps/docs/sidebars.ts apps/docs/README.md justfile package.json Cargo.toml', SRS-03:start:end -->
- [x] Local setup docs point to HTTP-hosted local model services and `ollama:<model>`. [SRS-03/AC-02] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "ollama:<model>|ollama:qwen3" README.md CONFIGURATION.md apps/docs/docs/start-here/first-turn.mdx apps/docs/docs/concepts/model-routing.mdx && rg -n "local HTTP model service|Local HTTP model client|model process is outside Paddles" README.md CONFIGURATION.md apps/docs/docs/start-here/first-turn.mdx apps/docs/docs/concepts/model-routing.mdx apps/docs/src/pages/index.tsx', SRS-03:start:end -->
- [x] Sift retrieval/indexing documentation, if still present, is clearly separated from model inference. [SRS-04/AC-03] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && rg -n "Sift remains.*retrieval|sift-direct.*retrieval|sift.*retrieves|retrieval/indexing" README.md ARCHITECTURE.md CONFIGURATION.md apps/docs/docs/concepts/search-retrieval.mdx apps/docs/docs/concepts/model-routing.mdx && ! rg -n -i "sift.*inference|sift_agent|sift_planner|qwen-1.5b" README.md ARCHITECTURE.md CONFIGURATION.md apps/docs/docs apps/docs/src', SRS-04:start:end -->
