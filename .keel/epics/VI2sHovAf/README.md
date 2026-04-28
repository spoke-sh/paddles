---
# system-managed
id: VI2sHovAf
created_at: 2026-04-27T18:36:24
# authored
title: Stream Uncut Tool Output
index: 66
mission: VI2q5DKHe
---

# Stream Uncut Tool Output

> Shell, inspect, and other tool outputs are captured to process::Output and trimmed to 1,200 chars by trim_for_planner in src/application/planner_action_execution.rs, which silently slices real cargo build, pytest, grep, and git log output before the planner sees it and makes long commands look frozen in the TUI. Stream output to operator and planner as bytes arrive; raise any planner-bound budget to 32k+ with head+tail truncation; keep raw output uncut in the trace recorder.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 0/1 voyages complete, 0/1 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Stream Tool Output And Drop The 1.2k Cap](voyages/VI2seFPac/) | planned | 0/1 |
<!-- END GENERATED -->
