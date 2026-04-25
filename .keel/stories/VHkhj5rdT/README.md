---
# system-managed
id: VHkhj5rdT
status: done
created_at: 2026-04-24T16:01:34
updated_at: 2026-04-24T17:45:18
# authored
title: Define External Capability Broker Port And Catalog
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgG2aro
index: 1
started_at: 2026-04-24T17:42:31
completed_at: 2026-04-24T17:45:18
---

# Define External Capability Broker Port And Catalog

## Summary

Define the runtime broker port and capability catalog needed to replace the noop external capability broker without forcing network access.

## Acceptance Criteria

- [x] A broker registry exposes declared external capability availability through a domain/application boundary. [SRS-01/AC-01] <!-- verify: cargo test external_capability_broker -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The default catalog remains unavailable unless local configuration enables a capability. [SRS-NFR-01/AC-01] <!-- verify: cargo test external_capability_default_posture -- --nocapture, SRS-NFR-01:start:end, proof: ac-2.log-->
