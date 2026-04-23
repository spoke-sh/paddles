---
# system-managed
id: VHaVRREzt
status: done
created_at: 2026-04-22T22:10:02
updated_at: 2026-04-22T23:20:21
# authored
title: Publish Consumer-Facing Paddles Projection Payloads
type: feat
operator-signal:
scope: VHaTau3dH/VHaTcsMav
index: 3
started_at: 2026-04-22T23:15:24
submitted_at: 2026-04-22T23:20:15
completed_at: 2026-04-22T23:20:21
---

# Publish Consumer-Facing Paddles Projection Payloads

## Summary

Publish the replay-derived projection payloads that downstream consumers can
use for transcript/detail rendering and deterministic restore, with Transit
rather than HTTP acting as the canonical projection surface.

## Acceptance Criteria

- [x] The projection contract publishes transcript rows, turn status, replay revision metadata, and trace/manifold availability in a typed consumer-facing payload. [SRS-03/AC-01] <!-- verify: cargo test consumer_projection_payloads_include_transcript_status_and_revision_metadata -- --nocapture, SRS-03:start:end, proof: ac-1.log-->
- [x] Projection tests prove the payload remains replay-derived and Transit-canonical rather than reconstructed from web-session state. [SRS-04/AC-02] <!-- verify: cargo test transit_projection_payloads_remain_replay_derived -- --nocapture, SRS-04:start:end, proof: ac-2.log-->
- [x] Projection payloads remain replay-derived views over authoritative Transit history rather than ad hoc web-session state. [SRS-NFR-02/AC-03] <!-- verify: cargo test consumer_projection_payloads_remain_replay_derived_views -- --nocapture, SRS-NFR-02:start:end, proof: ac-3.log-->
- [x] The versioned stream families, payload invariants, and compatibility expectations are documented with the published projection contract. [SRS-05/AC-04] <!-- verify: manual, SRS-05:start:end, proof: ac-4.log-->
- [x] Contract tests cover the public envelopes and reject unsupported or malformed versions without requiring UI scraping. [SRS-NFR-01/AC-05] <!-- verify: cargo test hosted_transit_contract_rejects_unsupported_versions -- --nocapture, SRS-NFR-01:start:end, proof: ac-5.log-->
