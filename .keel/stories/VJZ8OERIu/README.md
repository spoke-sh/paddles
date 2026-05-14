---
# system-managed
id: VJZ8OERIu
status: icebox
created_at: 2026-05-13T21:29:58
updated_at: 2026-05-13T21:36:48
# authored
title: Purge Local Model Loading Documentation
type: docs
operator-signal:
scope: VJZ034dF2/VJZ8DqFnJ
index: 3
---

# Purge Local Model Loading Documentation

## Summary

Purge documentation that teaches paddles-owned local inference model loading.
Docs should direct local-first users to HTTP-hosted model services and the
`ollama:<model>` provider form.

## Acceptance Criteria

- [ ] README, ARCHITECTURE, CONFIGURATION, POLICY, and build notes no longer describe paddles-owned local inference model loading as supported behavior. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Local setup docs point to HTTP-hosted local model services and `ollama:<model>`. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [ ] Sift retrieval/indexing documentation, if still present, is clearly separated from model inference. [SRS-04/AC-03] <!-- verify: manual, SRS-04:start:end -->
