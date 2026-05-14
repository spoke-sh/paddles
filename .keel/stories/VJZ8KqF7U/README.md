---
# system-managed
id: VJZ8KqF7U
status: backlog
created_at: 2026-05-13T21:29:45
updated_at: 2026-05-13T21:36:09
# authored
title: Preserve Sift Retrieval Outside Inference Cleanup
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8CYrLb
index: 3
---

# Preserve Sift Retrieval Outside Inference Cleanup

## Summary

Protect Sift retrieval/indexing from the inference cleanup. The story should
prove retrieval remains separately selectable and does not depend on removed
model-provider behavior.

## Acceptance Criteria

- [ ] Tests prove legacy Sift model-provider branches fail before runtime construction using the ADR compatibility policy. [SRS-03/AC-01] <!-- verify: automated, SRS-03:start:end -->
- [ ] Tests prove Sift retrieval/indexing can be prepared without Sift model-provider inference paths. [SRS-04/AC-02] <!-- verify: automated, SRS-04:start:end -->
- [ ] Retrieval provider selection remains independent from action-selection and final-rendering model-client selection. [SRS-04/AC-03] <!-- verify: automated, SRS-04:start:end -->
- [ ] Any inference cleanup that would require deleting retrieval/indexing is stopped and split into a later mission decision. [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
