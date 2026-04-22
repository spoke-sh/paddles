---
# system-managed
id: VHXK92haY
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T10:04:28
# authored
title: Add Limited Or No-Op Deliberation Modes For Inception Ollama And Sift
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipyBc
index: 2
started_at: 2026-04-22T09:59:56
completed_at: 2026-04-22T10:04:28
---

# Add Limited Or No-Op Deliberation Modes For Inception Ollama And Sift

## Summary

Model the providers that do not expose reusable native reasoning continuity as
explicit limited, summary-only, toggle-only, or unsupported/no-op paths so the
runtime degrades deliberately instead of silently.

## Acceptance Criteria

- [x] Inception, Ollama, and Sift advertise explicit limited or no-op deliberation behavior rather than pretending to support native continuity. [SRS-05/AC-01] <!-- verify: cargo test capability_surface_classifies_deliberation_support_for_runtime_provider_paths -- --nocapture, SRS-05:start:end -->
- [x] Unsupported combinations fail soft through explicit capability reporting and fallback semantics. [SRS-NFR-01/AC-02] <!-- verify: cargo test ollama_deliberation_support_tracks_thinking_family_through_tags_and_namespaces -- --nocapture, SRS-NFR-01:start:end -->
