---
# system-managed
id: VHXK92haY
status: backlog
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:14:06
# authored
title: Add Limited Or No-Op Deliberation Modes For Inception Ollama And Sift
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipyBc
index: 2
---

# Add Limited Or No-Op Deliberation Modes For Inception Ollama And Sift

## Summary

Model the providers that do not expose reusable native reasoning continuity as
explicit limited, summary-only, toggle-only, or unsupported/no-op paths so the
runtime degrades deliberately instead of silently.

## Acceptance Criteria

- [ ] Inception, Ollama, and Sift advertise explicit limited or no-op deliberation behavior rather than pretending to support native continuity. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end -->
- [ ] Unsupported combinations fail soft through explicit capability reporting and fallback semantics. [SRS-NFR-01/AC-02] <!-- verify: manual, SRS-NFR-01:start:end -->
