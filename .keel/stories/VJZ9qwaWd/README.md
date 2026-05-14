---
# system-managed
id: VJZ9qwaWd
status: icebox
created_at: 2026-05-13T21:35:47
updated_at: 2026-05-13T21:38:18
# authored
title: Preserve HTTP Provider Credential Rules
type: feat
operator-signal:
scope: VJZ034dF2/VJZ8DAKbC
index: 4
---

# Preserve HTTP Provider Credential Rules

## Summary

Preserve HTTP provider credential and availability behavior while the preference
schema changes. Optional local providers such as Ollama should remain usable
without credentials, while credentialed HTTP providers fail closed when required
secrets are missing.

## Acceptance Criteria

- [ ] Tests prove optional Ollama-style local HTTP providers remain available without credentials. [SRS-04/AC-01] <!-- verify: automated, SRS-04:start:end -->
- [ ] Tests prove required HTTP providers fail closed with provider-specific credential guidance when secrets are missing. [SRS-04/AC-02] <!-- verify: automated, SRS-04:start:end -->
- [ ] Preference migration does not bypass existing credential-store or transport-mediator boundaries. [SRS-04/AC-03] <!-- verify: automated, SRS-04:start:end -->
