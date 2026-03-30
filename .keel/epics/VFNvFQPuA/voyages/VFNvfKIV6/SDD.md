# Interpretation Visibility - Software Design Description

> Surface structured interpretation context in the TUI transcript with tiered verbosity

**SRS:** [SRS.md](SRS.md)

## Overview

Enrich the TurnEvent payload emitted after interpretation context assembly so the TUI has the structured data for tiered rendering. Add a separate event for guidance graph expansion. Render at three verbosity tiers without additional model calls.

## Components

### Enriched TurnEvent::InterpretationContext

Add fields: `doc_count: usize`, `hint_count: usize`, `procedure_count: usize`, `detail: String`. Populated from InterpretationContext at emission in application/mod.rs (~line 1031).

### Tiered format_turn_event_row

Pass verbose level to rendering. Produces:
- v=0: `"2 docs, 3 hints, 1 procedure from AGENTS.md, INSTRUCTIONS.md"`
- v=1: Above + source names with first-line excerpt previews, tool hint summaries
- v=2: Full `InterpretationContext::render()` output

### TurnEvent::GuidanceGraphExpanded

New variant from `expand_interpretation_guidance_graph` in sift_agent.rs. Fields: `docs_discovered`, `depth_reached`, `root_sources`. min_verbosity = 1.

## Data Flow

1. `derive_interpretation_context` → application/mod.rs populates enriched event
2. Event → TurnEventSink → UiMessage → handle_message → format_turn_event_row at verbose tier
3. Graph expansion event emitted separately from sift_agent, same flow

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Pass verbose to format_turn_event_row | Yes | Keeps events data-only, rendering is TUI concern |
| Full render at v=2 | InterpretationContext::render() | Reuses existing code |
