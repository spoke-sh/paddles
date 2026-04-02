---
# system-managed
id: VFfvB0L8b
status: done
created_at: 2026-04-02T15:25:52
updated_at: 2026-04-02T16:00:03
# authored
title: Bootstrap A Tested React Runtime Web App
type: feat
operator-signal:
scope: VFfuuVwYJ/VFfvAz07R
index: 3
started_at: 2026-04-02T15:56:03
submitted_at: 2026-04-02T15:59:59
completed_at: 2026-04-02T16:00:03
---

# Bootstrap A Tested React Runtime Web App

## Summary

Create the runtime React app boundary that will progressively absorb the embedded web shell. This slice focuses on app scaffolding, route ownership, and tests.

## Acceptance Criteria

- [x] A React runtime web app exists with route scaffolding for `/`, `/transit`, and `/manifold`. [SRS-03/AC-01] <!-- verify: nix develop --command sh -lc 'npm run test --workspace @paddles/web && npm run e2e --workspace @paddles/web', SRS-03:start:end, proof: ac-1.log-->
- [x] The runtime React app includes automated unit/integration coverage and browser E2E coverage. [SRS-04/AC-02] <!-- verify: nix develop --command sh -lc 'npm run test --workspace @paddles/web && npm run e2e --workspace @paddles/web', SRS-04:start:end, proof: ac-2.log-->
- [x] The app layout and modules reduce long-term duplication and prepare later route migration into React. [SRS-NFR-03/AC-03] <!-- verify: manual, SRS-NFR-03:start:end, proof: ac-3.log-->
