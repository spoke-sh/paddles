---
# system-managed
id: VFc2myzhw
status: backlog
created_at: 2026-04-01T23:31:01
updated_at: 2026-04-01T23:37:25
# authored
title: Support Inception Edit-Native Endpoints
type: feat
operator-signal:
scope: VFc2hwU7e/VFc2jHVLG
index: 5
---

# Support Inception Edit-Native Endpoints

## Summary

Add a dedicated follow-on slice for Inception’s edit-native endpoints so
coder/edit behavior can be integrated intentionally, with its own transport and
UX decisions, instead of being hidden inside the basic chat-provider bring-up.

## Acceptance Criteria

- [ ] The plan preserves a dedicated slice for edit-native endpoints separate from the chat-completions provider integration [SRS-05/AC-01]. <!-- verify: board, SRS-05:start:end -->
- [ ] The slice explicitly protects the Mercury-2 compatibility path from depending on edit-native endpoint work [SRS-NFR-03/AC-02]. <!-- verify: board, SRS-NFR-03:start:end -->
