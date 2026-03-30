---
# system-managed
id: VFNvkxVgP
status: icebox
created_at: 2026-03-30T13:35:16
updated_at: 2026-03-30T13:35:16
# authored
title: Emit Guidance Graph Expansion Event
type: feat
operator-signal:
scope: VFNvFQPuA/VFNvfKIV6
index: 3
---

# Emit Guidance Graph Expansion Event

## Summary

Add a new TurnEvent::GuidanceGraphExpanded variant emitted after expand_interpretation_guidance_graph completes. Carries docs_discovered, depth_reached, root_sources. Wire emission through the event sink, add TUI rendering, assign min_verbosity=1.

## Acceptance Criteria

- [ ] TurnEvent::GuidanceGraphExpanded exists with docs_discovered, depth_reached, root_sources fields [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Event emitted from sift_agent.rs after graph expansion completes [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [ ] format_turn_event_row renders it as a human-readable line [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] min_verbosity=1; hidden at default, visible at -v and above [SRS-03/AC-04] <!-- verify: manual, SRS-03:start:end -->
