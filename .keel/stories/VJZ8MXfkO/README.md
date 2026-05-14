---
# system-managed
id: VJZ8MXfkO
status: backlog
created_at: 2026-05-13T21:29:51
updated_at: 2026-05-13T21:36:11
# authored
title: Document Ollama Local HTTP Defaults
type: docs
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 3
---

# Document Ollama Local HTTP Defaults

## Summary

Update configuration and setup docs so local-first inference is documented as
an HTTP-hosted model service, with Ollama examples using `ollama:<model>`.

## Acceptance Criteria

- [ ] `CONFIGURATION.md` documents turn-runtime preference precedence and the new preference shape. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] Local inference examples use `ollama:<model>` without naming a fixed default model. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [ ] Docs no longer describe runtime lanes as the canonical provider preference model. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
