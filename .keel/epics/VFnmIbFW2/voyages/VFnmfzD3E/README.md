---
# system-managed
id: VFnmfzD3E
status: done
epic: VFnmIbFW2
created_at: 2026-04-03T23:42:17
# authored
title: Emit and Render Applied Edit Diffs
index: 1
updated_at: 2026-04-03T23:44:17
started_at: 2026-04-03T23:49:05
completed_at: 2026-04-04T00:16:54
---

# Emit and Render Applied Edit Diffs

> Show applied workspace changes as first-class diff artifacts across the runtime, web UI, and TUI so workspace editor agency is obvious.

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
**Progress:** 4/4 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Emit Structured Applied Edit Artifacts From The Workspace Editor](../../../../stories/VFnmpXLWV/README.md) | feat | done |
| [Render Applied Edit Diffs In The Web Runtime Stream](../../../../stories/VFnmpYoYK/README.md) | feat | done |
| [Render Applied Edit Diffs In The TUI Transcript Stream](../../../../stories/VFnmpaHZS/README.md) | feat | done |
| [Lock Diff Visibility With Projection And Contract Tests](../../../../stories/VFnmpbfZe/README.md) | feat | done |
<!-- END GENERATED -->

## Retrospective

**What went well:** Shared applied-edit artifacts now render consistently across runtime, web, and TUI.

**What was harder than expected:** Story verification only became submit-ready after replacing placeholder commands with real proof commands.

**What would you do differently:** Author concrete verification commands in story scaffolds instead of defaulting to test.

