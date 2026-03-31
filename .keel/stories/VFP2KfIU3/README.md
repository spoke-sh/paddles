---
# system-managed
id: VFP2KfIU3
status: icebox
created_at: 2026-03-30T18:07:28
updated_at: 2026-03-30T18:54:01
# authored
title: Implement ContextLocator Tier Metadata
type: feat
operator-signal:
scope: VFOmY0WHC/VFOvKhUFc
index: 2
---

# Implement ContextLocator Tier Metadata

## Summary

Ensure locators carry tier metadata.

## Acceptance Criteria

- [ ] ContextLocator includes tier field [SRS-02/AC-01] <!-- verify: test, SRS-02:start:end -->
- [ ] ArtifactEnvelope carries ContextLocator with tier [SRS-03/AC-01] <!-- verify: test, SRS-03:start:end -->
- [ ] No leakage of transit/sift types in ports [SRS-NFR-02/AC-01] <!-- verify: test, SRS-NFR-02:start:end -->
