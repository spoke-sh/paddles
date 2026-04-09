---
# system-managed
id: VGGIuWOhH
status: done
created_at: 2026-04-08T20:45:56
updated_at: 2026-04-08T21:21:38
# authored
title: Remove Transit Chrome In Favor Of Machine Narrative
type: feat
operator-signal:
scope: VGGIor3dC/VGGIqtM2e
index: 3
started_at: 2026-04-08T21:16:31
submitted_at: 2026-04-08T21:21:36
completed_at: 2026-04-08T21:21:38
---

# Remove Transit Chrome In Favor Of Machine Narrative

## Summary

Strip away redundant transit chrome and controls once the new machine stage already explains the turn clearly.

## Acceptance Criteria

- [x] Legacy transit controls or cards that duplicate the machine narrative are removed, reduced, or moved behind an internals path. [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: ac-1.log-->
- [x] Transit route tests are updated to guard the simpler operator path rather than the older chrome-heavy surface. [SRS-NFR-02/AC-02] <!-- verify: manual, SRS-NFR-02:start:end, proof: ac-2.log-->
