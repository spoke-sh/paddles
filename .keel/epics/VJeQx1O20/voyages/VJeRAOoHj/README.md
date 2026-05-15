---
# system-managed
id: VJeRAOoHj
status: done
epic: VJeQx1O20
created_at: 2026-05-14T19:15:55
# authored
title: Unify First Action Entry Point
index: 1
updated_at: 2026-05-14T19:19:26
started_at: 2026-05-14T19:22:08
completed_at: 2026-05-14T19:52:51
---

# Unify First Action Entry Point

> Normal turn execution enters execute_agent_loop before any model-selected action; the first loop iteration handles direct answers, stops, and workspace actions.

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
**Progress:** 2/2 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Route First Action Through Agent Loop](../../../../stories/VJeRUd3hl/README.md) | feat | done |
| [Collapse Initial Action Interface](../../../../stories/VJeRVkb73/README.md) | refactor | done |
<!-- END GENERATED -->

## Retrospective

**What went well:** First-action routing now enters the recursive agent loop, and the legacy initial-action selector was removed from the runtime action-selection interface.

**What was harder than expected:** Some legacy compatibility tests still describe first-action behavior while the production path no longer calls a separate selector.

**What would you do differently:** Continue collapsing compatibility helpers in later stories so test vocabulary follows the runtime contract more closely.

