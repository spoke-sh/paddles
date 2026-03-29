---
# system-managed
id: VFDMsjvYs
status: done
created_at: 2026-03-28T18:14:01
updated_at: 2026-03-28T18:48:16
# authored
title: Require Grounded Synthesis With Default File Citations
type: feat
operator-signal:
scope: VFDMnu8k9/VFDMp3Zn3
index: 3
started_at: 2026-03-28T18:46:56
submitted_at: 2026-03-28T18:48:13
completed_at: 2026-03-28T18:48:16
---

# Require Grounded Synthesis With Default File Citations

## Summary

Constrain repository-question synthesis to answer from explicit evidence bundles,
cite source files by default, and say when the available evidence is too weak
to support a confident answer.

## Acceptance Criteria

- [x] Repository-question answers are synthesized from evidence bundles and include source/file citations by default. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end, proof: ac-1.log-->
- [x] When evidence is missing or insufficient, the answer says so explicitly instead of improvising unsupported repository claims. [SRS-04/AC-02] <!-- verify: manual, SRS-04:start:end, proof: ac-2.log-->
- [x] Tests or transcript proofs show both grounded cited answers and insufficient-evidence behavior. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end, proof: ac-3.log-->
