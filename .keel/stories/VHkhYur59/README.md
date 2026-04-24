---
# system-managed
id: VHkhYur59
status: backlog
created_at: 2026-04-24T16:00:55
updated_at: 2026-04-24T16:04:40
# authored
title: Seed Recursive Harness Eval Corpus
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgOF9KK
index: 2
---

# Seed Recursive Harness Eval Corpus

## Summary

Seed the eval suite with the first recursive harness scenarios covering evidence gathering, denied or degraded tools, edit obligations, delegation, context pressure, and replay.

## Acceptance Criteria

- [ ] The eval corpus includes initial scenarios for recursive evidence, tool recovery, edit obligations, delegation, context pressure, and replay. [SRS-02/AC-01] <!-- verify: cargo test eval_corpus -- --nocapture, SRS-02:start:end -->
- [ ] Eval failures identify the violated harness contract instead of only returning a generic failure. [SRS-NFR-02/AC-01] <!-- verify: cargo test eval_failure_reporting -- --nocapture, SRS-NFR-02:start:end -->
