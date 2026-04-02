---
# system-managed
id: VFfvAzz7F
status: done
created_at: 2026-04-02T15:25:52
updated_at: 2026-04-02T16:04:12
# authored
title: Stage Route Cutover Between Embedded Shell And React App
type: feat
operator-signal:
scope: VFfuuVwYJ/VFfvAz07R
index: 2
started_at: 2026-04-02T16:02:05
submitted_at: 2026-04-02T16:04:05
completed_at: 2026-04-02T16:04:12
---

# Stage Route Cutover Between Embedded Shell And React App

## Summary

Define and implement the controlled cutover seam between the existing embedded shell and the React runtime app so migration can happen without breaking current operator workflows.

## Acceptance Criteria

- [x] The first React slice preserves the existing Rust backend API surface and keeps the embedded shell available until parity work is complete. [SRS-05/AC-01] <!-- verify: nix develop --command sh -lc 'cargo test -q infrastructure::web::tests::web_router_serves_dedicated_manifold_and_transit_routes && npm run e2e --workspace @paddles/web', SRS-05:start:end, proof: ac-1.log-->
- [x] Repo documentation states clearly that the embedded shell remains the runtime source of truth until React route cutover is complete. [SRS-NFR-02/AC-02] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-2.log-->
