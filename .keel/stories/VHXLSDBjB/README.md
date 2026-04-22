---
# system-managed
id: VHXLSDBjB
status: backlog
created_at: 2026-04-22T09:11:33
updated_at: 2026-04-22T09:14:06
# authored
title: Expose Thinking Modes Across Supported Providers
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipyBc
index: 4
---

# Expose Thinking Modes Across Supported Providers

## Summary

Expose supported thinking modes, reasoning effort controls, or explicit `none`
results across every supported provider/model path so runtime selection and
configuration no longer depend on provider-specific hard-coding.

## Acceptance Criteria

- [ ] Every supported provider/model path exposes supported thinking modes or an explicit `none`/unsupported result through provider catalogs and runtime configuration. [SRS-07/AC-01] <!-- verify: manual, SRS-07:start:end -->
- [ ] Thinking-mode catalogs stay synchronized with the actual provider capability surface and fallback behavior. [SRS-NFR-03/AC-02] <!-- verify: manual, SRS-NFR-03:start:end -->
