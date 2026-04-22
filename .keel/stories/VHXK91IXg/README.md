---
# system-managed
id: VHXK91IXg
status: backlog
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:14:06
# authored
title: Record Debug-Scoped Deliberation Artifacts Without Polluting Rationale
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipEBj
index: 2
---

# Record Debug-Scoped Deliberation Artifacts Without Polluting Rationale

## Summary

Add the bounded debug or forensic recording path for provider-native reasoning
artifacts so maintainers can inspect continuity behavior without contaminating
canonical turn records or paddles-authored rationale.

## Acceptance Criteria

- [ ] Provider-native reasoning artifacts, if recorded, live on a debug-scoped path separate from canonical transcript/render persistence. [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] Contract tests cover one native-continuation provider and one explicit no-op provider. [SRS-05/AC-02] <!-- verify: manual, SRS-05:start:end -->
