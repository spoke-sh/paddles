---
# system-managed
id: VJZ9qwaWd
status: done
created_at: 2026-05-13T21:35:47
updated_at: 2026-05-13T22:25:07
# authored
title: Preserve HTTP Provider Credential Rules
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 4
started_at: 2026-05-13T22:21:19
completed_at: 2026-05-13T22:25:07
---

# Preserve HTTP Provider Credential Rules

## Summary

Preserve HTTP provider credential and availability behavior while the preference
schema changes. Optional local providers such as Ollama should remain usable
without credentials, while credentialed HTTP providers fail closed when required
secrets are missing.

## Acceptance Criteria

- [x] Tests prove optional Ollama-style local HTTP providers remain available without credentials. [SRS-04/AC-01] <!-- verify: cargo nextest run optional_local_provider_stays_enabled_without_credentials mediator_allows_optional_ollama_provider_without_credentials ollama_model_clients_build_without_credentials, SRS-04:start:end -->
- [x] Tests prove required HTTP providers fail closed with provider-specific credential guidance when secrets are missing. [SRS-04/AC-02] <!-- verify: cargo nextest run required_remote_provider_is_disabled_when_missing_credentials mediator_fails_closed_for_missing_required_provider_credentials, SRS-04:start:end -->
- [x] Preference migration does not bypass existing credential-store or transport-mediator boundaries. [SRS-04/AC-03] <!-- verify: cargo nextest run legacy_runtime_lane_preferences_migrate_into_turn_runtime_shape migrated_turn_runtime_preferences_do_not_bypass_transport_mediator_credentials, SRS-04:start:end -->
