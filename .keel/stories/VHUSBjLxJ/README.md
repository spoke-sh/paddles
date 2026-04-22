---
# system-managed
id: VHUSBjLxJ
status: backlog
created_at: 2026-04-21T21:19:29
updated_at: 2026-04-21T21:24:11
# authored
title: Remove Nested Tool Loops From Model Adapters
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS5RqZf
index: 3
---

# Remove Nested Tool Loops From Model Adapters

## Summary

Retire adapter-owned repository tool loops so Sift, HTTP, and related model
integrations no longer compete with the application recursive harness for
budgets, retries, and stop ownership.

## Acceptance Criteria

- [ ] Model adapters stop running independent repository tool loops once the application harness owns execution control. [SRS-03/AC-01] <!-- verify: test, SRS-03:start:end -->
- [ ] Budget, retry, stop, and governance events for repository actions are emitted from the application recursive loop only. [SRS-04/AC-02] <!-- verify: review, SRS-04:start:end -->
