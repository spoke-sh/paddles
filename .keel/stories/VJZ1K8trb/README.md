---
# system-managed
id: VJZ1K8trb
status: backlog
created_at: 2026-05-13T21:01:54
updated_at: 2026-05-13T21:03:32
# authored
title: Inventory Sift Model Inference Surfaces
type: chore
operator-signal:
scope: VJZ0tpZQJ/VJZ14yp0U
index: 1
---

# Inventory Sift Model Inference Surfaces

## Summary

Create the source-backed inventory of in-process Sift model-provider,
model-preparation, and local inference surfaces. The output should distinguish
Sift-as-model-provider from Sift-as-retrieval-backend so future deletion work
does not remove useful indexing behavior accidentally.

## Acceptance Criteria

- [ ] Inventory lists Sift model-provider and model-loading files, tests, CLI/config references, and docs that future implementation must migrate or delete. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Inventory classifies each Sift reference as model inference, model preparation, retrieval/indexing, compatibility alias, test fixture, or documentation. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end -->
- [ ] Inventory identifies initial red/green test anchors for removing paddles-owned local model loading. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end -->
