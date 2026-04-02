---
# system-managed
id: VFesYmgZk
status: done
created_at: 2026-04-02T11:09:12
updated_at: 2026-04-02T11:52:32
# authored
title: Link Manifold States Back To Forensic Sources And Document The Route
type: feat
operator-signal:
scope: VFes0Rhaj/VFes287sD
index: 6
started_at: 2026-04-02T11:50:53
completed_at: 2026-04-02T11:52:32
---

# Link Manifold States Back To Forensic Sources And Document The Route

## Summary

Keep the manifold accountable to the exact system by linking selections back to precise forensic sources and documenting the route clearly. This slice covers source drilldown, route-to-route navigation, and the foundational/public docs that explain what the manifold is and is not.

## Acceptance Criteria

- [x] Selecting a manifold state reveals exact underlying sources and supports navigation back to the precise forensic inspector [SRS-07/AC-01] <!-- verify: cargo test -q manifold_route_html_links_selected_sources_back_to_forensics && cargo test -q infrastructure::web::tests && cargo check -q, SRS-07:start:end -->
- [x] Foundational and public docs describe the manifold route, steering signal semantics, and the metaphorical limits of the visualization [SRS-08/AC-02] <!-- verify: rg -n "Web Trace Routes|manifold route is metaphorical|Forensic Inspector And Manifold Route|accountability limits|locally served web shell" /home/alex/workspace/spoke-sh/paddles/README.md /home/alex/workspace/spoke-sh/paddles/ARCHITECTURE.md /home/alex/workspace/spoke-sh/paddles/website/docs/concepts/context-pressure.mdx /home/alex/workspace/spoke-sh/paddles/website/docs/reference/foundational-docs.mdx, SRS-08:start:end -->
- [x] Any new visualization dependency remains locally served or vendored and does not violate the local-first contract [SRS-NFR-03/AC-03] <!-- verify: sh -lc '! rg -n "react-force-graph|three\\.js|@react-three|\\bcobe\\b|unpkg|cdn\\.jsdelivr|src=\\\"https://|href=\\\"https://" /home/alex/workspace/spoke-sh/paddles/src/infrastructure/web/index.html /home/alex/workspace/spoke-sh/paddles/Cargo.toml /home/alex/workspace/spoke-sh/paddles/README.md /home/alex/workspace/spoke-sh/paddles/ARCHITECTURE.md /home/alex/workspace/spoke-sh/paddles/website/docs', SRS-NFR-03:start:end -->
