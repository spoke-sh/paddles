---
# system-managed
id: VHUSBjLxJ
status: done
created_at: 2026-04-21T21:19:29
updated_at: 2026-04-21T22:31:07
# authored
title: Remove Nested Tool Loops From Model Adapters
type: refactor
operator-signal:
scope: VHURpL4nG/VHUS5RqZf
index: 3
started_at: 2026-04-21T22:17:08
completed_at: 2026-04-21T22:31:07
---

# Remove Nested Tool Loops From Model Adapters

## Summary

Retire adapter-owned repository tool loops so Sift, HTTP, and related model
integrations no longer compete with the application recursive harness for
budgets, retries, and stop ownership.

## Acceptance Criteria

- [x] Model adapters stop running independent repository tool loops once the application harness owns execution control. [SRS-03/AC-01] <!-- verify: cargo test infrastructure::adapters::sift_agent::tests -- --nocapture, SRS-03:start:end, proof: ac-1.log -->
- [x] Budget, retry, stop, and governance events for repository actions are emitted from the application recursive loop only. [SRS-04/AC-02] <!-- verify: /home/alex/workspace/spoke-sh/paddles/.keel/stories/VHUSBjLxJ/EVIDENCE/verify-ac-2.sh, SRS-04:start:end, proof: ac-2.log -->
