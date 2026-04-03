---
# system-managed
id: VFguXXCOf
status: backlog
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T19:32:06
# authored
title: Run Full Browser E2E In Just Test And Governor Verification
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 3
---

# Run Full Browser E2E In Just Test And Governor Verification

## Summary

Make the browser suite part of the real protection path by ensuring `just test` and the pre-commit governor both run the full product-route E2E under the same environment contract. Use this slice to align workflow docs with the simplified architecture as well.

## Acceptance Criteria

- [ ] `just test` runs the full product-route browser suite needed to protect the cross-surface projection contract [SRS-07/AC-01] <!-- verify: test, SRS-07:start:end -->
- [ ] The governor path exercises the same browser verification contract rather than a reduced or environment-divergent subset [SRS-NFR-04/AC-02] <!-- verify: test, SRS-NFR-04:start:end -->
- [ ] Foundational and public docs describe the simplified projection architecture, route ownership, and verification model accurately [SRS-08/AC-03] <!-- verify: review, SRS-08:start:end -->
