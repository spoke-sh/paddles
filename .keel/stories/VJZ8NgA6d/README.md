---
# system-managed
id: VJZ8NgA6d
status: icebox
created_at: 2026-05-13T21:29:56
updated_at: 2026-05-13T21:36:46
# authored
title: Remove Inference-Only Sift Model Dependencies
type: chore
operator-signal:
scope: VJZ034dF2/VJZ8DqFnJ
index: 2
---

# Remove Inference-Only Sift Model Dependencies

## Summary

Remove inference-only dependencies and build surfaces that become unused after
the Sift inference adapters are gone. Retrieval dependencies should remain only
when retrieval code still needs them.

## Acceptance Criteria

- [ ] Dependency review identifies Candle/Qwen/tokenizer or other inference-only crates that no remaining active code uses. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Cargo/build configuration removes inference-only dependencies that are no longer needed. [SRS-02/AC-02] <!-- verify: automated, SRS-02:start:end -->
- [ ] HTTP provider and Sift retrieval tests remain green after dependency cleanup. [SRS-NFR-01/AC-03] <!-- verify: automated, SRS-NFR-01:start:end -->
