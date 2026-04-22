---
# system-managed
id: VHUSDTFaN
status: backlog
created_at: 2026-04-21T21:19:36
updated_at: 2026-04-21T21:24:11
# authored
title: Move Runtime Event Presentation Out Of The Domain Model
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS6H0Kd
index: 3
---

# Move Runtime Event Presentation Out Of The Domain Model

## Summary

Relocate runtime event formatting and surface-oriented projectors out of the
domain model so domain events remain presentation-free while TUI and web keep
receiving equivalent projected data.

## Acceptance Criteria

- [ ] Runtime event presentation and projector logic move out of `domain/model` into a non-domain boundary. [SRS-04/AC-01] <!-- verify: review, SRS-04:start:end -->
- [ ] Domain events remain usable without surface-specific strings while TUI and web continue to receive equivalent presentation data through the new boundary. [SRS-NFR-02/AC-02] <!-- verify: test, SRS-NFR-02:start:end -->
