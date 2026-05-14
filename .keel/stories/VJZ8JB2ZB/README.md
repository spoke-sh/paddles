---
# system-managed
id: VJZ8JB2ZB
status: backlog
created_at: 2026-05-13T21:29:39
updated_at: 2026-05-13T21:36:07
# authored
title: Codify Legacy Sift Provider Migration Failure
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8Bws9Z
index: 2
---

# Codify Legacy Sift Provider Migration Failure

## Summary

Codify the compatibility behavior for old Sift model-provider settings. Legacy
Sift inference config must fail explicitly with an actionable migration hint
instead of silently changing providers.

## Acceptance Criteria

- [ ] Tests prove `provider = "sift"` and equivalent planner/final-rendering legacy provider selections fail before runtime construction. [SRS-02/AC-01] <!-- verify: automated, SRS-02:start:end -->
- [ ] The failure message states that `sift` no longer performs model inference and tells the operator to choose an HTTP provider such as `ollama:<model>`. [SRS-02/AC-02] <!-- verify: automated, SRS-02:start:end -->
- [ ] Sift retrieval/indexing selections are not rejected by this model-provider compatibility policy. [SRS-02/AC-03] <!-- verify: automated, SRS-02:start:end -->
