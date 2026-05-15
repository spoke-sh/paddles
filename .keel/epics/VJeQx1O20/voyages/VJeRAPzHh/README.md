---
# system-managed
id: VJeRAPzHh
status: done
epic: VJeQx1O20
created_at: 2026-05-14T19:15:55
# authored
title: Move Turn Contract Into Agent Loop
index: 2
updated_at: 2026-05-14T19:19:31
started_at: 2026-05-14T19:53:49
completed_at: 2026-05-14T20:13:05
---

# Move Turn Contract Into Agent Loop

> Read-only, execution, review, edit, commit, and grounding policy are loop inputs and execution-contract constraints rather than pre-loop routing decisions.

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
| [Rename Collaboration Runtime Contract](../../../../stories/VJeRX3FXi/README.md) | refactor | done |
| [Move Turn Obligations Into Loop State](../../../../stories/VJeRYVkyL/README.md) | refactor | done |
<!-- END GENERATED -->

## Retrospective

**What went well:** Turn contract policy and turn obligations now enter the recursive loop as request state and completion-contract data.

**What was harder than expected:** Untangling old test-only bootstrap fixtures from the newer model-selected loop path without losing mutation guard coverage.

**What would you do differently:** Keep future loop policy changes centered on AgentLoopState first, then project them into prompts and execution contracts.

