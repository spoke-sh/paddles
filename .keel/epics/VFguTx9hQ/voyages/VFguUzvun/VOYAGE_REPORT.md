# VOYAGE REPORT: Unified Projection Store And Product-Route Sync

## Voyage Metadata
- **ID:** VFguUzvun
- **Epic:** VFguTx9hQ
- **Status:** done
- **Goal:** -

## Execution Summary
**Progress:** 6/6 stories complete

## Implementation Narrative
### Serve A Unified Web Bootstrap And Projection Event Stream
- **ID:** VFguXWMOh
- **Status:** done

#### Summary
Replace the current fan-out of panel-specific bootstrap and live event paths with one web bootstrap response and one session-scoped projection event stream. This slice makes the browser hydrate and stay live from one contract instead of coordinating multiple fetch/SSE surfaces.

#### Acceptance Criteria
- [x] The web adapter exposes one bootstrap endpoint that returns the canonical conversation projection for the shared session [SRS-02/AC-01] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q shared_bootstrap_route_returns_shared_session_projection -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] The web adapter exposes one session-scoped live projection stream that replaces panel-specific event ownership and remains replay-recoverable [SRS-02/AC-02] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q broadcast_event_sink_tags_turn_events_with_the_session_projection_identity -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
- [x] The new bootstrap/live contracts preserve replay as the authoritative recovery path after missed updates [SRS-NFR-01/AC-03] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q broadcast_projection_sinks_rebuild_snapshots_from_authoritative_replay -- --nocapture, SRS-NFR-01:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFguXWMOh/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFguXWMOh/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFguXWMOh/EVIDENCE/ac-3.log)

### Expose Canonical Conversation Projection Snapshots And Updates
- **ID:** VFguXWiOg
- **Status:** done

#### Summary
Define one application-facing conversation projection contract that packages transcript, forensic, manifold, and transit trace state for the shared interactive session. This slice should remove the need for the web runtime to stitch together panel-local read paths as separate sources of truth.

#### Acceptance Criteria
- [x] The application layer exposes a canonical conversation projection snapshot/update contract covering transcript, forensic, manifold, and trace graph state for a shared interactive session [SRS-01/AC-01] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q replay_conversation_projection_packages_all_shared_session_surfaces -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Projection updates derive from the same authoritative read models that back replay and remain sufficient for replay-backed rebuild after missed live updates [SRS-NFR-01/AC-02] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q conversation_projection_updates_are_derived_from_authoritative_replay_state -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFguXWiOg/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFguXWiOg/EVIDENCE/ac-2.log)

### Run Full Browser E2E In Just Test And Governor Verification
- **ID:** VFguXXCOf
- **Status:** done

#### Summary
Make the browser suite part of the real protection path by ensuring `just test` and the pre-commit governor both run the full product-route E2E under the same environment contract. Use this slice to align workflow docs with the simplified architecture as well.

#### Acceptance Criteria
- [x] `just test` runs the full product-route browser suite needed to protect the cross-surface projection contract [SRS-07/AC-01] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command just test, SRS-07:start:end, proof: ac-1.log-->
- [x] The governor path exercises the same browser verification contract rather than a reduced or environment-divergent subset [SRS-NFR-04/AC-02] <!-- verify: sh -lc 'cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q pre_commit_governor_runs_repo_quality_and_test_entrypoints -- --nocapture && cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q just_test_runs_frontend_workspace_test_checks -- --nocapture && cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q runtime_web_e2e_script_runs_the_product_route_playwright_suite -- --nocapture', SRS-NFR-04:start:end, proof: ac-2.log-->
- [x] Foundational and public docs describe the simplified projection architecture, route ownership, and verification model accurately [SRS-08/AC-03] <!-- verify: sh -lc 'test -s /home/alex/workspace/spoke-sh/paddles/.keel/stories/VFguXXCOf/EVIDENCE/ac-3.log', SRS-08:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFguXXCOf/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFguXXCOf/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFguXXCOf/EVIDENCE/ac-3.log)

### Replace The Raw Runtime Shell Bridge With A React Projection Store
- **ID:** VFguXXUPd
- **Status:** done

#### Summary
Remove the raw HTML runtime bridge and replace it with a real React-side projection store/hook that owns session hydration, live updates, and route-visible projection state. After this slice, TanStack should own runtime state instead of merely hosting an imperative shell.

#### Acceptance Criteria
- [x] The primary React runtime consumes the canonical projection through a shared projection store/hook instead of mounting raw HTML and global imperative bootstrap logic [SRS-03/AC-01] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command bash -lc 'cd /home/alex/workspace/spoke-sh/paddles/apps/web && npx vitest run src/runtime-app.test.tsx', SRS-03:start:end, proof: ac-1.log-->
- [x] Bootstrap, session identity, and live update handling are owned once inside the React runtime rather than duplicated across route hosts or bridge layers [SRS-NFR-03/AC-02] <!-- verify: sh -lc 'test -s /home/alex/workspace/spoke-sh/paddles/.keel/stories/VFguXXUPd/EVIDENCE/ac-2.log', SRS-NFR-03:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFguXXUPd/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFguXXUPd/EVIDENCE/ac-2.log)

### Port Chat Transit And Manifold Routes To TanStack With Visual Parity
- **ID:** VFguXXrPr
- **Status:** done

#### Summary
Port the current chat, transit trace, and manifold surfaces into TanStack routes that render from the shared projection store while preserving the current operator-facing design and route behavior exactly. The goal is ownership simplification without visual or behavioral drift.

#### Acceptance Criteria
- [x] `/`, `/transit`, and `/manifold` render from the shared projection store and preserve the current route semantics and operator workflow [SRS-04/AC-01] <!-- verify: sh -lc 'test -s /home/alex/workspace/spoke-sh/paddles/.keel/stories/VFguXXrPr/EVIDENCE/ac-1.log', SRS-04:start:end, proof: ac-1.log-->
- [x] The cutover preserves the current layout, typography, controls, and behavior closely enough to avoid design drift without a separate human decision [SRS-NFR-02/AC-02] <!-- verify: sh -lc 'test -s /home/alex/workspace/spoke-sh/paddles/.keel/stories/VFguXXrPr/EVIDENCE/ac-2.log', SRS-NFR-02:start:end, proof: ac-2.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFguXXrPr/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFguXXrPr/EVIDENCE/ac-2.log)

### Add Cross-Surface Product-Route E2E With External Turn Injection
- **ID:** VFguXYDR8
- **Status:** done

#### Summary
Add the browser test the product actually needs: keep the web page open, inject a turn from outside the page against the shared conversation session, and prove that transcript, transit, and manifold update live and survive reload on the product routes.

#### Acceptance Criteria
- [x] Product-route browser E2E keeps a page open and verifies that an externally injected turn appears live in transcript, transit, and manifold without reload [SRS-05/AC-01] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command npm --workspace @paddles/web run e2e, SRS-05:start:end, proof: ac-1.log-->
- [x] Browser E2E verifies route continuity and replay-backed recovery after reload for the same shared conversation session [SRS-06/AC-02] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command npm --workspace @paddles/web run e2e, SRS-06:start:end, proof: ac-2.log-->
- [x] The suite exercises the actual product routes rather than alternate or legacy-only route families [SRS-NFR-04/AC-03] <!-- verify: nix develop /home/alex/workspace/spoke-sh/paddles --command npm --workspace @paddles/web run e2e, SRS-NFR-04:start:end, proof: ac-3.log-->

#### Verified Evidence
- [ac-1.log](../../../../stories/VFguXYDR8/EVIDENCE/ac-1.log)
- [ac-2.log](../../../../stories/VFguXYDR8/EVIDENCE/ac-2.log)
- [ac-3.log](../../../../stories/VFguXYDR8/EVIDENCE/ac-3.log)


