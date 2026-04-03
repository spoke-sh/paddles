---
# system-managed
id: VFguXXrPr
status: backlog
created_at: 2026-04-02T19:29:37
updated_at: 2026-04-02T19:32:06
# authored
title: Port Chat Transit And Manifold Routes To TanStack With Visual Parity
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 5
---

# Port Chat Transit And Manifold Routes To TanStack With Visual Parity

## Summary

Port the current chat, transit trace, and manifold surfaces into TanStack routes that render from the shared projection store while preserving the current operator-facing design and route behavior exactly. The goal is ownership simplification without visual or behavioral drift.

## Acceptance Criteria

- [ ] `/`, `/transit`, and `/manifold` render from the shared projection store and preserve the current route semantics and operator workflow [SRS-04/AC-01] <!-- verify: manual, SRS-04:start:end -->
- [ ] The cutover preserves the current layout, typography, controls, and behavior closely enough to avoid design drift without a separate human decision [SRS-NFR-02/AC-02] <!-- verify: manual, SRS-NFR-02:start:end -->
