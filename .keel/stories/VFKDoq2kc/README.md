---
# system-managed
id: VFKDoq2kc
status: needs-human-verification
created_at: 2026-03-29T22:21:55
updated_at: 2026-03-29T22:29:27
# authored
title: Axum Server With Health And SSE Turn Streaming
type: feat
operator-signal:
scope: VFKBCVjpo/VFKDkffWL
index: 1
started_at: 2026-03-29T22:24:20
submitted_at: 2026-03-29T22:29:27
---

# Axum Server With Health And SSE Turn Streaming

## Summary

Add an axum HTTP server to paddles that starts alongside the CLI/TUI, serving session lifecycle, turn submission, and SSE-streamed TurnEvents. The server is an infrastructure adapter sharing the existing MechSuitService instance.

## Acceptance Criteria

- [x] Axum server starts on configurable port with --port flag [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [x] GET /health returns 200 with runtime lane config [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [x] POST /sessions creates session and returns ID [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [x] POST /sessions/:id/turns processes prompt and returns response [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
- [x] GET /sessions/:id/events streams TurnEvents as SSE [SRS-05/AC-05] <!-- verify: manual, SRS-05:start:end -->
- [x] TurnEvent derives Serialize [SRS-06/AC-06] <!-- verify: manual, SRS-06:start:end -->
