---
# system-managed
id: VFGy8zKU8
status: backlog
created_at: 2026-03-29T09:00:51
updated_at: 2026-03-29T09:05:52
# authored
title: Preserve Graph Episode State In Evidence And Turn Events
type: feat
operator-signal:
scope: VFGy53NJt/VFGy6j0OE
index: 2
---

# Preserve Graph Episode State In Evidence And Turn Events

## Summary

Map the richer upstream graph episode/frontier/branch state into typed
`paddles` metadata so graph-mode gatherers can surface useful branch-local
evidence, graph stop reasons, and concise operator-visible telemetry without
leaking raw `sift` internals through the domain boundary.

## Acceptance Criteria

- [ ] Graph-mode gatherer results preserve typed graph episode/frontier/branch metadata and graph stop reasons in the gathered evidence bundle. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
- [ ] The metadata boundary remains `paddles`-owned rather than exposing raw upstream `sift` graph DTOs across the domain. [SRS-NFR-02/AC-03] <!-- verify: manual, SRS-NFR-02:start:end -->
