---
# system-managed
id: VFNvkwbgF
status: backlog
created_at: 2026-03-30T13:35:16
updated_at: 2026-03-30T14:19:09
# authored
title: Tiered TUI Rendering For Interpretation Context
type: feat
operator-signal:
scope: VFNvFQPuA/VFNvfKIV6
index: 2
---

# Tiered TUI Rendering For Interpretation Context

## Summary

Update format_turn_event_row for InterpretationContext to render tiered detail. Default: category breakdown line. -v: document sources with excerpt previews, tool hint summaries. -vv: full InterpretationContext::render() output. Requires passing verbose level into the rendering path.

## Acceptance Criteria

- [ ] Default verbosity renders a single category breakdown line with counts and source names [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] At -v, document source names with first-line excerpt previews and tool hint summaries are shown [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] At -vv, full InterpretationContext::render() output is displayed [SRS-02/AC-03] <!-- verify: manual, SRS-02:start:end -->
- [ ] Verbose level is accessible within the rendering path [SRS-02/AC-04] <!-- verify: manual, SRS-02:start:end -->
- [ ] When all category counts are zero, a meaningful empty state is shown [SRS-02/AC-05] <!-- verify: manual, SRS-02:start:end -->
