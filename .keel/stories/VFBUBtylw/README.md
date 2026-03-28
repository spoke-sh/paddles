---
# system-managed
id: VFBUBtylw
status: done
created_at: 2026-03-28T10:30:30
updated_at: 2026-03-28T13:12:38
# authored
title: Define Context-Gathering Subagent Contract
type: feat
operator-signal:
scope: VFBTXlHli/VFBTYpPo6
index: 1
started_at: 2026-03-28T13:07:32
submitted_at: 2026-03-28T13:12:35
completed_at: 2026-03-28T13:12:38
---

# Define Context-Gathering Subagent Contract

## Summary

Define the internal gatherer request/result contract, evidence bundle shape, and
foundational documentation needed to separate context gathering from final
answer synthesis.

## Acceptance Criteria

- [x] A typed context-gathering request/result contract exists and can represent ranked evidence, synthesis-ready summaries, and explicit gatherer capability states. [SRS-01/AC-01] <!-- verify: manual, SRS-01:start:end, proof: ac-1.log-->
- [x] Foundational docs describe the gatherer vs synthesizer split and the evidence bundle boundary in a way that future adapters can implement without guessing intent. [SRS-01/AC-02] <!-- verify: manual, SRS-01:start:end, proof: ac-2.log-->
- [x] The contract is synthesis-oriented and does not force the gatherer to pretend it produced the final user-facing answer. [SRS-01/AC-03] <!-- verify: manual, SRS-01:start:end, proof: ac-3.log-->
