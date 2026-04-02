---
# system-managed
id: VFc2mxZgj
status: done
created_at: 2026-04-01T23:31:01
updated_at: 2026-04-02T06:02:18
# authored
title: Add Inception Provider Catalog And Authentication Support
type: feat
operator-signal:
scope: VFc2hwU7e/VFc2jHVLG
index: 1
started_at: 2026-04-02T05:59:25
completed_at: 2026-04-02T06:02:18
---

# Add Inception Provider Catalog And Authentication Support

## Summary

Add `Inception` as a first-class remote provider in the provider catalog,
credential store, and operator-facing selection surfaces so paddles can
authenticate and present `mercury-2` as a selectable model before any runtime
integration work.

## Acceptance Criteria

- [x] `ModelProvider`, provider availability, and credential resolution recognize `Inception` with the correct base URL, auth requirement, and `INCEPTION_API_KEY` wiring [SRS-01/AC-01]. <!-- verify: cargo test -q auth_requirements_distinguish_local_optional_and_required_providers, SRS-01:start:end, proof: ac-1.log-->
- [x] `/login inception` and `/model` can distinguish authenticated versus unauthenticated Inception states without regressing other providers [SRS-01/AC-02]. <!-- verify: cargo test -q model_command_lists_enabled_and_disabled_provider_catalog_entries, SRS-01:start:end, proof: ac-2.log-->
- [x] Missing Inception credentials fail closed while existing provider selection behavior remains intact [SRS-NFR-01/AC-03]. <!-- verify: cargo test -q required_remote_provider_is_disabled_when_missing_credentials, SRS-NFR-01:start:end, proof: ac-3.log-->
