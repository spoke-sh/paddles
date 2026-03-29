---
# system-managed
id: VFDMr1Sum
status: backlog
created_at: 2026-03-28T18:13:54
updated_at: 2026-03-28T18:17:19
# authored
title: Make Gatherer The Default Path For Repo Questions
type: feat
operator-signal:
scope: VFDMnu8k9/VFDMp3Zn3
index: 1
---

# Make Gatherer The Default Path For Repo Questions

## Summary

Route repository-question turns through the explicit gatherer boundary by
default and stop relying on hidden synthesizer-private retrieval as the primary
repo-answer path.

## Acceptance Criteria

- [ ] Repository-question turns use the configured gatherer lane by default when one is available, and the controller no longer treats hidden synthesizer-private retrieval as the normal repo-answer path. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] When the gatherer lane is unavailable or fails, the controller/runtime selects a clearly labeled fallback path instead of silently pretending the same gatherer-backed path ran. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] Tests or CLI proofs cover both gatherer-present and gatherer-missing repo-question behavior. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
