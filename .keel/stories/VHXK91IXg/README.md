---
# system-managed
id: VHXK91IXg
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:39:47
# authored
title: Record Debug-Scoped Deliberation Artifacts Without Polluting Rationale
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipEBj
index: 2
started_at: 2026-04-22T09:35:23
completed_at: 2026-04-22T09:39:47
---

# Record Debug-Scoped Deliberation Artifacts Without Polluting Rationale

## Summary

Add the bounded debug or forensic recording path for provider-native reasoning
artifacts so maintainers can inspect continuity behavior without contaminating
canonical turn records or paddles-authored rationale.

## Acceptance Criteria

- [x] Provider-native reasoning artifacts, if recorded, live on a debug-scoped path separate from canonical transcript/render persistence. [SRS-04/AC-01] <!-- verify: cargo test moonshot_reasoning_artifacts_record_on_forensic_debug_path_only -- --nocapture, SRS-04:start:end -->
- [x] Contract tests cover one native-continuation provider and one explicit no-op provider. [SRS-05/AC-02] <!-- verify: cargo test openai_toggle_only_models_do_not_emit_deliberation_artifacts -- --nocapture, SRS-05:start:end -->
