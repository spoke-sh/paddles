---
# system-managed
id: VFguXWiOg
status: done
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T20:38:16
# authored
title: Expose Canonical Conversation Projection Snapshots And Updates
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 2
started_at: 2026-04-02T19:53:25
completed_at: 2026-04-02T20:38:16
---

# Expose Canonical Conversation Projection Snapshots And Updates

## Summary

Define one application-facing conversation projection contract that packages transcript, forensic, manifold, and transit trace state for the shared interactive session. This slice should remove the need for the web runtime to stitch together panel-local read paths as separate sources of truth.

## Acceptance Criteria

- [x] The application layer exposes a canonical conversation projection snapshot/update contract covering transcript, forensic, manifold, and trace graph state for a shared interactive session [SRS-01/AC-01] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q replay_conversation_projection_packages_all_shared_session_surfaces -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Projection updates derive from the same authoritative read models that back replay and remain sufficient for replay-backed rebuild after missed live updates [SRS-NFR-01/AC-02] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q conversation_projection_updates_are_derived_from_authoritative_replay_state -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->
