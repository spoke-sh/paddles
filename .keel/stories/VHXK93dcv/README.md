---
# system-managed
id: VHXK93dcv
status: backlog
created_at: 2026-04-22T09:06:21
updated_at: 2026-04-22T09:14:06
# authored
title: Use Deliberation Signals In Recursive Branch Refine And Stop Decisions
type: feat
operator-signal:
scope: VHXJWQaFC/VHXJiqMD1
index: 2
---

# Use Deliberation Signals In Recursive Branch Refine And Stop Decisions

## Summary

Wire normalized deliberation signals into the recursive harness so the planner
can make better continue, branch, refine, retry, and stop decisions without
matching on provider-native payloads.

## Acceptance Criteria

- [ ] The recursive harness uses normalized deliberation signals to improve branch, refine, retry, and stop decisions. [SRS-03/AC-01] <!-- verify: manual, SRS-03:start:end -->
