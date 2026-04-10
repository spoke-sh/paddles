---
# system-managed
id: VGLDQCIrB
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T18:50:00
# authored
title: Model Optional Specialist Brains Without Breaking The Recursive Planner
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMvU4i
index: 3
started_at: 2026-04-09T18:40:27
completed_at: 2026-04-09T18:50:00
---

# Model Optional Specialist Brains Without Breaking The Recursive Planner

## Summary

Model optional specialist brains as bounded session-scoped capabilities rather than alternate architectures. This story should protect the recursive planner/controller core while allowing future auxiliary brains to plug in cleanly.

## Acceptance Criteria

- [x] Optional specialist brains plug into the same session and capability contracts instead of bypassing the recursive planner/controller architecture [SRS-03/AC-01] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test specialist_brain -- --nocapture', SRS-03:start:end, proof: ac-1.log -->
- [x] The design keeps fallback behavior clear when a specialist brain is absent or unsupported for the active profile/model shape [SRS-03/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && cargo test planner_requests_include_specialist_brain_runtime_notes -- --nocapture && cargo test planner_requests_include_specialist_brain_fallback_for_prompt_envelope_safe_profiles -- --nocapture && rg -n "session-continuity-v1|Specialist brains|specialist-brain ids|query_session_context" README.md ARCHITECTURE.md CONFIGURATION.md src/application/mod.rs src/infrastructure/harness_profile.rs src/infrastructure/specialist_brains.rs', SRS-03:start:end, proof: ac-2.log -->
