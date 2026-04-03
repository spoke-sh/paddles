---
# system-managed
id: VFguXXCOf
status: done
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T20:41:10
# authored
title: Run Full Browser E2E In Just Test And Governor Verification
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 3
started_at: 2026-04-02T20:32:36
completed_at: 2026-04-02T20:41:10
---

# Run Full Browser E2E In Just Test And Governor Verification

## Summary

Make the browser suite part of the real protection path by ensuring `just test` and the pre-commit governor both run the full product-route E2E under the same environment contract. Use this slice to align workflow docs with the simplified architecture as well.

## Acceptance Criteria

- [x] `just test` runs the full product-route browser suite needed to protect the cross-surface projection contract [SRS-07/AC-01] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command just test, SRS-07:start:end, proof: ac-1.log-->
- [x] The governor path exercises the same browser verification contract rather than a reduced or environment-divergent subset [SRS-NFR-04/AC-02] <!-- verify: sh -lc 'cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q pre_commit_governor_runs_repo_quality_and_test_entrypoints -- --nocapture && cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q just_test_runs_frontend_workspace_test_checks -- --nocapture && cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q runtime_web_e2e_script_runs_the_product_route_playwright_suite -- --nocapture', SRS-NFR-04:start:end, proof: ac-2.log-->
- [x] Foundational and public docs describe the simplified projection architecture, route ownership, and verification model accurately [SRS-08/AC-03] <!-- verify: sh -lc 'test -s /home/alex/workspace/spoke-sh/paddles/.keel/stories/VFguXXCOf/EVIDENCE/ac-3.log', SRS-08:start:end, proof: ac-3.log-->
