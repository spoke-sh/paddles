---
# system-managed
id: VGLDQ92nP
status: backlog
created_at: 2026-04-09T16:55:30
updated_at: 2026-04-09T16:58:15
# authored
title: Replace Provider Branching With Negotiated Capability Surface
type: feat
operator-signal:
scope: VGLD4Iesy/VGLDMuE5W
index: 3
---

# Replace Provider Branching With Negotiated Capability Surface

## Summary

Replace provider-name-specific runtime branching with a negotiated capability surface for shared planner, renderer, and tool-call behavior. This story should define the contract rather than finalizing every migration.

## Acceptance Criteria

- [ ] Shared planner/render/tool-call behavior resolves from capability descriptors wherever the behavior is conceptually common across providers [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] The capability surface is documented and testable enough that future providers can fit the harness without forking controller logic [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
