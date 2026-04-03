---
# system-managed
id: VFguXXUPd
status: done
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T20:40:19
# authored
title: Replace The Raw Runtime Shell Bridge With A React Projection Store
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 4
started_at: 2026-04-02T20:32:36
completed_at: 2026-04-02T20:40:19
---

# Replace The Raw Runtime Shell Bridge With A React Projection Store

## Summary

Remove the raw HTML runtime bridge and replace it with a real React-side projection store/hook that owns session hydration, live updates, and route-visible projection state. After this slice, TanStack should own runtime state instead of merely hosting an imperative shell.

## Acceptance Criteria

- [x] The primary React runtime consumes the canonical projection through a shared projection store/hook instead of mounting raw HTML and global imperative bootstrap logic [SRS-03/AC-01] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command bash -lc 'cd /home/alex/workspace/spoke-sh/paddles/apps/web && npx vitest run src/runtime-app.test.tsx', SRS-03:start:end, proof: ac-1.log-->
- [x] Bootstrap, session identity, and live update handling are owned once inside the React runtime rather than duplicated across route hosts or bridge layers [SRS-NFR-03/AC-02] <!-- verify: sh -lc 'test -s /home/alex/workspace/spoke-sh/paddles/.keel/stories/VFguXXUPd/EVIDENCE/ac-2.log', SRS-NFR-03:start:end, proof: ac-2.log-->
