---
# system-managed
id: VHXK90tXf
status: done
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:25:37
# authored
title: Define Deliberation Capability Surface And Adapter State
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJipEBj
index: 1
started_at: 2026-04-22T09:14:11
completed_at: 2026-04-22T09:25:37
---

# Define Deliberation Capability Surface And Adapter State

## Summary

Define the provider-agnostic deliberation capability surface and the opaque
adapter state shape that later provider implementations will plug into, without
changing canonical transcript/render state or replacing paddles `rationale`.

## Acceptance Criteria

- [x] Provider/model negotiation classifies deliberation support explicitly for every provider path needed by the runtime. [SRS-01/AC-01] <!-- verify: cargo test capability_surface_ -- --nocapture, SRS-01:start:end -->
- [x] Adapter turn interfaces can return and accept opaque deliberation state separately from authored response and paddles rationale. [SRS-02/AC-02] <!-- verify: cargo test provider_turn_request_and_response_keep_deliberation_state_separate_from_content -- --nocapture, SRS-02:start:end -->
