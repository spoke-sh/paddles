---
# system-managed
id: VFc2my2gi
status: backlog
created_at: 2026-04-01T23:31:01
updated_at: 2026-04-01T23:37:25
# authored
title: Expose Inception Defaults And Operator Guidance
type: chore
operator-signal:
scope: VFc2hwU7e/VFc2jHVLG
index: 3
---

# Expose Inception Defaults And Operator Guidance

## Summary

Document how operators should authenticate and select Inception, identify
`mercury-2` as the core supported model, and make the difference between core
compatibility and optional native capabilities explicit.

## Acceptance Criteria

- [ ] README/configuration guidance explains how to authenticate and select Inception with the supported core model path [SRS-03/AC-01]. <!-- verify: manual, SRS-03:start:end -->
- [ ] Operator guidance distinguishes the Mercury-2 compatibility slice from the optional streaming/diffusion and edit-native slices [SRS-03/AC-02]. <!-- verify: manual, SRS-03:start:end -->
- [ ] The guidance does not imply that optional native capabilities are required before the provider is usable [SRS-NFR-03/AC-03]. <!-- verify: manual, SRS-NFR-03:start:end -->
