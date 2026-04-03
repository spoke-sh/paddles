---
# system-managed
id: VFguXWMOh
status: done
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T20:38:31
# authored
title: Serve A Unified Web Bootstrap And Projection Event Stream
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 1
started_at: 2026-04-02T20:32:36
completed_at: 2026-04-02T20:38:31
---

# Serve A Unified Web Bootstrap And Projection Event Stream

## Summary

Replace the current fan-out of panel-specific bootstrap and live event paths with one web bootstrap response and one session-scoped projection event stream. This slice makes the browser hydrate and stay live from one contract instead of coordinating multiple fetch/SSE surfaces.

## Acceptance Criteria

- [x] The web adapter exposes one bootstrap endpoint that returns the canonical conversation projection for the shared session [SRS-02/AC-01] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q shared_bootstrap_route_returns_shared_session_projection -- --nocapture, SRS-02:start:end, proof: ac-1.log-->
- [x] The web adapter exposes one session-scoped live projection stream that replaces panel-specific event ownership and remains replay-recoverable [SRS-02/AC-02] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q broadcast_event_sink_tags_turn_events_with_the_session_projection_identity -- --nocapture, SRS-02:start:end, proof: ac-2.log-->
- [x] The new bootstrap/live contracts preserve replay as the authoritative recovery path after missed updates [SRS-NFR-01/AC-03] <!-- verify: cargo test --manifest-path /home/alex/workspace/spoke-sh/paddles/Cargo.toml -q broadcast_projection_sinks_rebuild_snapshots_from_authoritative_replay -- --nocapture, SRS-NFR-01:start:end, proof: ac-3.log-->
