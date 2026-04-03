---
# system-managed
id: VFguXWMOh
status: backlog
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T19:32:06
# authored
title: Serve A Unified Web Bootstrap And Projection Event Stream
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 1
---

# Serve A Unified Web Bootstrap And Projection Event Stream

## Summary

Replace the current fan-out of panel-specific bootstrap and live event paths with one web bootstrap response and one session-scoped projection event stream. This slice makes the browser hydrate and stay live from one contract instead of coordinating multiple fetch/SSE surfaces.

## Acceptance Criteria

- [ ] The web adapter exposes one bootstrap endpoint that returns the canonical conversation projection for the shared session [SRS-02/AC-01] <!-- verify: test, SRS-02:start:end -->
- [ ] The web adapter exposes one session-scoped live projection stream that replaces panel-specific event ownership and remains replay-recoverable [SRS-02/AC-02] <!-- verify: test, SRS-02:start:end -->
- [ ] The new bootstrap/live contracts preserve replay as the authoritative recovery path after missed updates [SRS-NFR-01/AC-03] <!-- verify: test, SRS-NFR-01:start:end -->
