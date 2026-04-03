---
# system-managed
id: VFguXYDR8
status: done
created_at: 2026-04-02T19:29:37
updated_at: 2026-04-02T20:39:24
# authored
title: Add Cross-Surface Product-Route E2E With External Turn Injection
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 6
started_at: 2026-04-02T20:32:36
completed_at: 2026-04-02T20:39:24
---

# Add Cross-Surface Product-Route E2E With External Turn Injection

## Summary

Add the browser test the product actually needs: keep the web page open, inject a turn from outside the page against the shared conversation session, and prove that transcript, transit, and manifold update live and survive reload on the product routes.

## Acceptance Criteria

- [x] Product-route browser E2E keeps a page open and verifies that an externally injected turn appears live in transcript, transit, and manifold without reload [SRS-05/AC-01] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command npm --workspace @paddles/web run e2e, SRS-05:start:end, proof: ac-1.log-->
- [x] Browser E2E verifies route continuity and replay-backed recovery after reload for the same shared conversation session [SRS-06/AC-02] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command npm --workspace @paddles/web run e2e, SRS-06:start:end, proof: ac-2.log-->
- [x] The suite exercises the actual product routes rather than alternate or legacy-only route families [SRS-NFR-04/AC-03] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command npm --workspace @paddles/web run e2e, SRS-NFR-04:start:end, proof: ac-3.log-->
