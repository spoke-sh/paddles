---
# system-managed
id: VFguXWiOg
status: backlog
created_at: 2026-04-02T19:29:36
updated_at: 2026-04-02T19:32:06
# authored
title: Expose Canonical Conversation Projection Snapshots And Updates
type: feat
operator-signal:
scope: VFguTx9hQ/VFguUzvun
index: 2
---

# Expose Canonical Conversation Projection Snapshots And Updates

## Summary

Define one application-facing conversation projection contract that packages transcript, forensic, manifold, and transit trace state for the shared interactive session. This slice should remove the need for the web runtime to stitch together panel-local read paths as separate sources of truth.

## Acceptance Criteria

- [ ] The application layer exposes a canonical conversation projection snapshot/update contract covering transcript, forensic, manifold, and trace graph state for a shared interactive session [SRS-01/AC-01] <!-- verify: test, SRS-01:start:end -->
- [ ] Projection updates derive from the same authoritative read models that back replay and remain sufficient for replay-backed rebuild after missed live updates [SRS-NFR-01/AC-02] <!-- verify: test, SRS-NFR-01:start:end -->
