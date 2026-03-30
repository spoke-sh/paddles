---
# system-managed
id: VFKN7LdAb
status: icebox
created_at: 2026-03-29T22:58:52
updated_at: 2026-03-29T22:58:52
# authored
title: Provider CLI Flag And Factory Routing
type: feat
operator-signal:
scope: VFKMjf28V/VFKN403tG
index: 1
---

# Provider CLI Flag And Factory Routing

## Summary

Add --provider CLI flag and route factory closures to the correct adapter constructor based on provider selection, with API key resolution from environment variables.

## Acceptance Criteria

- [ ] --provider flag accepts local, openai, anthropic, google, moonshot [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] Factory closures dispatch to correct adapter based on provider [SRS-02/AC-02] <!-- verify: manual, SRS-02:start:end -->
- [ ] API key resolved from provider-specific env var [SRS-03/AC-03] <!-- verify: manual, SRS-03:start:end -->
- [ ] --provider-url overrides default API base URL [SRS-04/AC-04] <!-- verify: manual, SRS-04:start:end -->
- [ ] Local provider is default when --provider omitted [SRS-05/AC-05] <!-- verify: manual, SRS-05:start:end -->
