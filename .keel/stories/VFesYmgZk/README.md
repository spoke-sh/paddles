---
# system-managed
id: VFesYmgZk
status: backlog
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:10:13
# authored
title: Link Manifold States Back To Forensic Sources And Document The Route
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 6
---

# Link Manifold States Back To Forensic Sources And Document The Route

## Summary

Keep the manifold accountable to the exact system by linking selections back to precise forensic sources and documenting the route clearly. This slice covers source drilldown, route-to-route navigation, and the foundational/public docs that explain what the manifold is and is not.

## Acceptance Criteria

- [ ] Selecting a manifold state reveals exact underlying sources and supports navigation back to the precise forensic inspector [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [ ] Foundational and public docs describe the manifold route, steering signal semantics, and the metaphorical limits of the visualization [SRS-08/AC-02] <!-- verify: review, SRS-08:start:end -->
- [ ] Any new visualization dependency remains locally served or vendored and does not violate the local-first contract [SRS-NFR-03/AC-03] <!-- verify: review, SRS-NFR-03:start:end -->
