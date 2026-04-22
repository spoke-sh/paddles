---
# system-managed
id: VHUSB4bI0
status: backlog
created_at: 2026-04-21T21:19:27
updated_at: 2026-04-21T21:24:11
# authored
title: Make Synthesizer Engines Author Responses Only
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS5RqZf
index: 2
---

# Make Synthesizer Engines Author Responses Only

## Summary

Trim the synthesizer boundary down to response authoring and synthesis-context
helpers so repository mutation is no longer part of the authoring contract.

## Acceptance Criteria

- [ ] `SynthesizerEngine` no longer exposes workspace mutation methods and remains responsible only for authored responses plus synthesis-context helpers. [SRS-02/AC-01] <!-- verify: review, SRS-02:start:end -->
- [ ] Existing turn flows continue to compile and route final response authoring through the new authoring-only contract. [SRS-NFR-02/AC-02] <!-- verify: test, SRS-NFR-02:start:end -->
