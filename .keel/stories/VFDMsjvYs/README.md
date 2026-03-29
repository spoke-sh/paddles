---
# system-managed
id: VFDMsjvYs
status: backlog
created_at: 2026-03-28T18:14:01
updated_at: 2026-03-28T18:17:19
# authored
title: Require Grounded Synthesis With Default File Citations
type: feat
operator-signal:
scope: VFDMnu8k9/VFDMp3Zn3
index: 3
---

# Require Grounded Synthesis With Default File Citations

## Summary

Constrain repository-question synthesis to answer from explicit evidence bundles,
cite source files by default, and say when the available evidence is too weak
to support a confident answer.

## Acceptance Criteria

- [ ] Repository-question answers are synthesized from evidence bundles and include source/file citations by default. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] When evidence is missing or insufficient, the answer says so explicitly instead of improvising unsupported repository claims. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end -->
- [ ] Tests or transcript proofs show both grounded cited answers and insufficient-evidence behavior. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
