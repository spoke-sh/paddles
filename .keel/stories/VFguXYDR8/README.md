---
# system-managed
id: VFguXYDR8
status: backlog
created_at: 2026-04-02T19:29:37
updated_at: 2026-04-02T19:32:06
# authored
title: Add Cross-Surface Product-Route E2E With External Turn Injection
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 6
---

# Add Cross-Surface Product-Route E2E With External Turn Injection

## Summary

Add the browser test the product actually needs: keep the web page open, inject a turn from outside the page against the shared conversation session, and prove that transcript, transit, and manifold update live and survive reload on the product routes.

## Acceptance Criteria

- [ ] Product-route browser E2E keeps a page open and verifies that an externally injected turn appears live in transcript, transit, and manifold without reload [SRS-05/AC-01] <!-- verify: test, SRS-05:start:end -->
- [ ] Browser E2E verifies route continuity and replay-backed recovery after reload for the same shared conversation session [SRS-06/AC-02] <!-- verify: test, SRS-06:start:end -->
- [ ] The suite exercises the actual product routes rather than alternate or legacy-only route families [SRS-NFR-04/AC-03] <!-- verify: test, SRS-NFR-04:start:end -->
