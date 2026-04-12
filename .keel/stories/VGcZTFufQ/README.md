---
# system-managed
id: VGcZTFufQ
status: backlog
created_at: 2026-04-12T16:09:43
updated_at: 2026-04-12T16:11:52
# authored
title: Define External Capability Contracts
type: feat
operator-signal:
scope: VGb1c1XAL/VGcZRpCKi
index: 1
---

# Define External Capability Contracts

## Summary

Define the typed capability-descriptor, invocation, result, and governance
contracts for web, MCP, and connector-backed actions so the recursive harness
can reason about external capability use through one vocabulary.

## Acceptance Criteria

- [ ] The runtime defines typed contracts for external capability descriptors, invocation intents and results, availability metadata, auth posture, and evidence expectations across web, MCP, and connector-backed actions. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end -->
- [ ] The external capability contract stays generic enough to absorb new fabrics without reworking the recursive planner contract. [SRS-NFR-04/AC-01] <!-- verify: manual, SRS-NFR-04:start:end -->
- [ ] The contract vocabulary composes with existing execution-governance and evidence-first boundaries instead of introducing surface-specific client paths. [SRS-NFR-03/AC-01] <!-- verify: manual, SRS-NFR-03:start:end -->
