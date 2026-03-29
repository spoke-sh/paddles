---
# system-managed
id: VFH4E7mBA
status: done
created_at: 2026-03-29T09:25:00
updated_at: 2026-03-29T09:57:18
# authored
title: Add TraceRecorder Port With Noop And In-Memory Adapters
type: feat
operator-signal:
scope: VFH4BXH4F/VFH4CCJ4d
index: 3
started_at: 2026-03-29T09:56:31
submitted_at: 2026-03-29T09:57:17
completed_at: 2026-03-29T09:57:18
---

# Add TraceRecorder Port With Noop And In-Memory Adapters

## Summary

Introduce a dedicated `TraceRecorder` port so durable turn recording stops
being conflated with transcript rendering, and prove the new boundary with
`noop` and in-memory implementations first.

## Acceptance Criteria

- [x] A `TraceRecorder` port exists separately from `TurnEventSink` and accepts the typed trace contract. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [x] `noop` and in-memory recorder adapters are available for local verification before storage-specific integration. [SRS-03/AC-02] <!-- verify: manual, SRS-03:start:end -->
- [x] Recorder failures degrade honestly without destabilizing live turn execution. [SRS-NFR-01/AC-03] <!-- verify: manual, SRS-NFR-01:start:end -->
