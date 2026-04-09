---
# system-managed
id: VGKoL95Nv
status: done
created_at: 2026-04-09T15:15:52
updated_at: 2026-04-09T15:52:27
# authored
title: Model Transport Configuration Auth And Diagnostics
type: feat
operator-signal:
scope: VGKnsYg1z/VGKoF0hsS
index: 2
started_at: 2026-04-09T15:45:44
completed_at: 2026-04-09T15:52:27
---

# Model Transport Configuration Auth And Diagnostics

## Summary

Model the authored configuration, auth, and diagnostics surfaces for native transports. This story should make enablement, bind targets, auth material, availability, and failure state visible through one shared operator-facing contract.

## Acceptance Criteria

- [x] The shared transport contract defines authored configuration and auth inputs for the named native transports without duplicating protocol-specific semantics [SRS-02/AC-01] <!-- verify: cargo test load_parses_native_transport_configuration_and_auth -- --nocapture, SRS-02:start:end -->
- [x] The shared diagnostics surface reports transport availability, negotiated mode, and latest failure details coherently enough for operators to inspect HTTP, SSE, WebSocket, and Transit through one model [SRS-02/AC-02] <!-- verify: cargo test health_route_reports_native_transport_diagnostics -- --nocapture && cargo test shared_bootstrap_route_returns_shared_session_projection -- --nocapture, SRS-02:start:end -->
