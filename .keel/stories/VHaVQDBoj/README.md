---
# system-managed
id: VHaVQDBoj
status: done
created_at: 2026-04-22T22:09:58
updated_at: 2026-04-22T23:09:49
# authored
title: Define Versioned Hosted Transit Envelopes And Stream Families
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcsMav
index: 1
started_at: 2026-04-22T23:07:03
completed_at: 2026-04-22T23:09:49
---

# Define Versioned Hosted Transit Envelopes And Stream Families

## Summary

Define the versioned hosted Transit stream families and envelope layout so
external clients can bootstrap sessions and submit turns over Transit instead
of depending on the web transport as the canonical boundary.

## Acceptance Criteria

- [x] The hosted Transit contract defines versioned envelopes for bootstrap, turn submission, progress, projection rebuild, completion/failure, and restore. [SRS-01/AC-01] <!-- verify: cargo test hosted_transit_contract_versions_ -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] The hosted Transit stream families provide the command/event/projection layout the runtime will build on in later stories. [SRS-01/AC-02] <!-- verify: cargo test hosted_transit_stream_families_define_runtime_layout -- --nocapture, SRS-01:start:end, proof: ac-2.log-->
