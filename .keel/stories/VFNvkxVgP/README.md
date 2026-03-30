---
# system-managed
id: VFNvkxVgP
status: done
updated_at: 2026-03-30T15:10:00
started_at: 2026-03-30T14:30:00
completed_at: 2026-03-30T15:10:00
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

- [x] TurnEvent::GuidanceGraphExpanded exists with docs_discovered, depth_reached, root_sources fields [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [x] Event emitted from sift_agent.rs after graph expansion completes [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [x] format_turn_event_row renders it as a human-readable line [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [x] Event emitted through existing event sink to TUI [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
- [x] min_verbosity=1; hidden at default, visible at -v and above [SRS-04/AC-05] <!-- verify: manual, SRS-04:start:end -->
