---
# system-managed
id: VF7tCKsgv
status: needs-human-verification
created_at: 2026-03-27T19:44:45
updated_at: 2026-03-27T23:13:58
# authored
title: Cut Over Runtime And Docs
type: feat
operator-signal:
scope: VF7t633ux/VF7tAvs7B
index: 3
started_at: 2026-03-27T23:13:21
submitted_at: 2026-03-27T23:13:58
---

# Cut Over Runtime And Docs

## Summary

Cut the CLI and repository boundary over to the new Sift-native runtime,
removing wonopcode from core execution and updating the authored docs.

## Acceptance Criteria

- [x] Single-prompt and interactive CLI flows remain operational after the cutover. [SRS-05/AC-01] <!-- verify: manual, SRS-05:start:end, proof: ac-1.log-->
- [x] wonopcode-core/provider/tools are removed from core runtime modules and Cargo runtime dependencies. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-2.log-->
- [x] Verbose output exposes context assembly and tool execution clearly enough to debug the controller. [SRS-NFR-02/AC-01] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-3.log-->
