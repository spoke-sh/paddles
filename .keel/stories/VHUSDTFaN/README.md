---
# system-managed
id: VHUSDTFaN
status: done
created_at: 2026-04-21T21:19:36
updated_at: 2026-04-21T23:04:43
# authored
title: Move Runtime Event Presentation Out Of The Domain Model
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS6H0Kd
index: 3
started_at: 2026-04-21T22:59:29
completed_at: 2026-04-21T23:04:43
---

# Move Runtime Event Presentation Out Of The Domain Model

## Summary

Relocate runtime event formatting and surface-oriented projectors out of the
domain model so domain events remain presentation-free while TUI and web keep
receiving equivalent projected data.

## Acceptance Criteria

- [x] Runtime event presentation and projector logic move out of `domain/model` into a non-domain boundary. [SRS-04/AC-01] <!-- verify: /home/alex/workspace/spoke-sh/paddles/.keel/stories/VHUSDTFaN/EVIDENCE/verify-review.sh, SRS-04:start:end, proof: review.md -->
- [x] Domain events remain usable without surface-specific strings while TUI and web continue to receive equivalent presentation data through the new boundary. [SRS-NFR-02/AC-02] <!-- verify: /home/alex/workspace/spoke-sh/paddles/.keel/stories/VHUSDTFaN/EVIDENCE/verify-ac-2.sh, SRS-NFR-02:start:end, proof: boundary-tests.log, proof: surface-tests.log -->
