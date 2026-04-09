---
# system-managed
id: VGGIuXjjA
status: backlog
created_at: 2026-04-08T20:45:57
updated_at: 2026-04-08T20:49:26
# authored
title: Build Forensic Machine Detail Drawer
type: feat
operator-signal:
scope: VGGIor3dC/VGGIqts2y
index: 3
---

# Build Forensic Machine Detail Drawer

## Summary

Build a focused forensic detail drawer that explains the selected machine moment, its steering forces, and any before/after artifact context.

## Acceptance Criteria

- [ ] The forensic route renders a machine-moment detail surface that explains why the selected moment mattered before exposing raw payloads. [SRS-02/AC-01] <!-- verify: manual, SRS-02:start:end -->
- [ ] Internals mode still exposes raw payloads, record ids, and evidence links without dominating the default detail presentation. [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] The detail surface exposes an explicit internals path for raw payloads, record ids, and comparison context without restoring the old always-on pane composition. [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
