---
# system-managed
id: VGGIqsj2g
status: done
epic: VGGIor3dC
created_at: 2026-04-08T20:45:42
# authored
title: Define Narrative Machine Model And Shared Projection
index: 1
updated_at: 2026-04-08T20:48:59
started_at: 2026-04-08T20:49:26
completed_at: 2026-04-08T21:04:19
---

# Define Narrative Machine Model And Shared Projection

> Define the simplified Rube Goldberg machine mental model, moment projection, and interaction contract so transit and forensic views share one causal vocabulary.

## Shared Vocabulary

This voyage pins the operator-facing machine vocabulary that later route stories must keep aligned:

- `Input`
- `Planner`
- `Evidence probe`
- `Diverter`
- `Jam`
- `Spring return`
- `Tool run`
- `Force`
- `Output`

These labels replace raw trace-storage language in the default path. Route rewrites may still expose raw records and payloads through internals, but they should not fall back to node/record-first copy for the main narrative.

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
**Progress:** 3/3 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Define Machine Moments And Shared Lexicon](../../../../stories/VGGIuTTeh/README.md) | feat | done |
| [Project Trace Records Into Narrative Machine Parts](../../../../stories/VGGIuUAef/README.md) | feat | done |
| [Guard Narrative Machine Contracts And Copy](../../../../stories/VGGIuUTee/README.md) | feat | done |
<!-- END GENERATED -->
