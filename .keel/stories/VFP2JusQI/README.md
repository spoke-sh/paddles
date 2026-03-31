---
# system-managed
id: VFP2JusQI
status: done
created_at: 2026-03-30T18:07:28
updated_at: 2026-03-30T20:48:37
# authored
title: Define ContextPressure And PressureFactor Types
type: feat
operator-signal:
scope: VFOmVwP8l/VFOvJSU8h
index: 1
started_at: 2026-03-30T20:25:00
completed_at: 2026-03-30T20:48:37
---

# Define ContextPressure And PressureFactor Types

## Summary

Define the domain types for tracking context pressure.

## Acceptance Criteria

- [x] ContextPressure struct definition [SRS-01/AC-01] <!-- verify: cargo test -- domain::model::context_quality::tests, SRS-01:start:end, proof: tests_passed.log -->
- [x] PressureLevel enum definition [SRS-02/AC-01] <!-- verify: cargo test -- domain::model::context_quality::tests, SRS-02:start:end, proof: tests_passed.log -->
- [x] PressureFactor enum definition [SRS-03/AC-01] <!-- verify: cargo test -- domain::model::context_quality::tests, SRS-03:start:end, proof: tests_passed.log -->
