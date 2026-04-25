---
# system-managed
id: VHkhv6S1q
status: done
created_at: 2026-04-24T16:02:20
updated_at: 2026-04-24T18:49:51
# authored
title: Add Local Session Store Contracts
type: feat
operator-signal:
scope: VHkfpJJc4/VHkgNakSc
index: 1
started_at: 2026-04-24T18:46:58
completed_at: 2026-04-24T18:49:51
---

# Add Local Session Store Contracts

## Summary

Add local-first session store contracts for turns, planner decisions, evidence, governance records, and execution posture.

## Acceptance Criteria

- [x] A session store port can persist and reload normalized turn, evidence, and governance records. [SRS-01/AC-01] <!-- verify: cargo test session_store_contract -- --nocapture, SRS-01:start:end, proof: ac-1.log-->
- [x] Stored records include schema or version metadata for future migrations. [SRS-NFR-02/AC-01] <!-- verify: cargo test session_store_versioning -- --nocapture, SRS-NFR-02:start:end, proof: ac-2.log-->
