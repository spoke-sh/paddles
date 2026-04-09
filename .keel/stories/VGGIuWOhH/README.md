---
# system-managed
id: VGGIuWOhH
status: backlog
created_at: 2026-04-08T20:45:56
updated_at: 2026-04-08T20:49:18
# authored
title: Remove Transit Chrome In Favor Of Machine Narrative
type: feat
operator-signal:
scope: VGGIor3dC/VGGIqtM2e
index: 3
---

# Remove Transit Chrome In Favor Of Machine Narrative

## Summary

Strip away redundant transit chrome and controls once the new machine stage already explains the turn clearly.

## Acceptance Criteria

- [ ] Legacy transit controls or cards that duplicate the machine narrative are removed, reduced, or moved behind an internals path. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end -->
- [ ] Transit route tests are updated to guard the simpler operator path rather than the older chrome-heavy surface. [SRS-NFR-02/AC-02] <!-- verify: manual, SRS-NFR-02:start:end -->
