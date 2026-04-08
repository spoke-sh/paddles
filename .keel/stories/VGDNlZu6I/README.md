---
# system-managed
id: VGDNlZu6I
status: done
created_at: 2026-04-08T08:46:25
updated_at: 2026-04-08T09:38:09
# authored
title: Fail Closed On Ambiguous Or Missing Entity Targets
type: feat
operator-signal:
scope: VGDNcabks/VGDNh30T9
index: 2
started_at: 2026-04-08T09:30:48
completed_at: 2026-04-08T09:38:09
---

# Fail Closed On Ambiguous Or Missing Entity Targets

## Summary

Fail closed when deterministic resolution cannot validate a safe authored target so ambiguous or missing entities produce explicit planner/runtime outcomes instead of malformed patches or off-boundary reads.

## Acceptance Criteria

- [x] Ambiguous or missing resolver outcomes prevent workspace mutation and surface a deterministic stop/fallback reason. [SRS-02/AC-01] <!-- verify: cargo nextest run unresolved_targets_fail_closed_before_workspace_mutation --no-tests pass, SRS-02:start:end, proof: ac-1.log-->
- [x] Non-authored or ignored targets remain rejected even when they appear in resolver candidates or planner hints. [SRS-NFR-01/AC-02] <!-- verify: cargo nextest run resolver_never_promotes_non_authored_targets --no-tests pass, SRS-NFR-01:start:end, proof: ac-2.log-->
