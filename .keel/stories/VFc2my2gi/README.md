---
# system-managed
id: VFc2my2gi
status: done
created_at: 2026-04-01T23:31:01
updated_at: 2026-04-02T06:13:33
# authored
title: Expose Inception Defaults And Operator Guidance
type: chore
operator-signal:
scope: VFc2hwU7e/VFc2jHVLG
index: 3
started_at: 2026-04-02T06:09:41
completed_at: 2026-04-02T06:13:33
---

# Expose Inception Defaults And Operator Guidance

## Summary

Document how operators should authenticate and select Inception, identify
`mercury-2` as the core supported model, and make the difference between core
compatibility and optional native capabilities explicit.

## Acceptance Criteria

- [x] README/configuration guidance explains how to authenticate and select Inception with the supported core model path [SRS-03/AC-01]. <!-- verify: cargo test -q readme_documents_inception_authentication_and_model_selection, SRS-03:start:end, proof: ac-1.log-->
- [x] Operator guidance distinguishes the Mercury-2 compatibility slice from the optional streaming/diffusion and edit-native slices [SRS-03/AC-02]. <!-- verify: cargo test -q configuration_guidance_distinguishes_core_inception_support_from_optional_capabilities, SRS-03:start:end, proof: ac-2.log-->
- [x] The guidance does not imply that optional native capabilities are required before the provider is usable [SRS-NFR-03/AC-03]. <!-- verify: cargo test -q configuration_guidance_marks_inception_core_path_as_immediately_usable, SRS-NFR-03:start:end, proof: ac-3.log-->
