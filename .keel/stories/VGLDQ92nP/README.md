---
# system-managed
id: VGLDQ92nP
status: done
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T17:24:04
# authored
title: Replace Provider Branching With Negotiated Capability Surface
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuE5W
index: 3
started_at: 2026-04-09T17:21:05
completed_at: 2026-04-09T17:24:04
---

# Replace Provider Branching With Negotiated Capability Surface

## Summary

Replace provider-name-specific runtime branching with a negotiated capability surface for shared planner, renderer, and tool-call behavior. This story should define the contract rather than finalizing every migration.

## Acceptance Criteria

- [x] Shared planner/render/tool-call behavior resolves from capability descriptors wherever the behavior is conceptually common across providers [SRS-03/AC-01] <!-- verify: cargo test capability_surface_negotiates_shared_http_render_and_tool_call_behavior -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] The capability surface is documented and testable enough that future providers can fit the harness without forking controller logic [SRS-03/AC-02] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && rg -n "capability surface|provider/model pair|planner tool-call|transport support|future provider|forking controller logic" README.md ARCHITECTURE.md CONFIGURATION.md src/infrastructure/providers.rs', SRS-03:start:end, proof: ac-2.log-->
