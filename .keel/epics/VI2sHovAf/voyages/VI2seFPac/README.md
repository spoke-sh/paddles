---
# system-managed
id: VI2seFPac
status: done
epic: VI2sHovAf
created_at: 2026-04-27T18:37:51
# authored
title: Stream Tool Output And Drop The 1.2k Cap
index: 1
updated_at: 2026-04-27T18:46:11
started_at: 2026-04-27T19:43:49
completed_at: 2026-04-27T19:47:05
---

# Stream Tool Output And Drop The 1.2k Cap

> Replace buffered process::Output capture in planner_action_execution.rs with streamed stdout/stderr pipes that fan out to TurnEventSink and the planner request as bytes arrive; remove the trim_for_planner(_, 1_200) cap; raise any planner-bound budget to 32k+ with head+tail truncation; keep raw output uncut in the trace recorder.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
| [VOYAGE_REPORT.md](VOYAGE_REPORT.md) | Narrative summary of implementation and evidence |
| [COMPLIANCE_REPORT.md](COMPLIANCE_REPORT.md) | Traceability matrix and verification proof |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 1/1 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Stream Planner Shell And Inspect Output](../../../../stories/VI2snS9cl/README.md) | feat | done |
<!-- END GENERATED -->
