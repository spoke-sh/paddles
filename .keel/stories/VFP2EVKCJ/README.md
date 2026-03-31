---
# system-managed
id: VFP2EVKCJ
status: done
created_at: 2026-03-30T18:07:16
updated_at: 2026-03-30T18:32:35
# authored
title: Implement ContextResolver Port And TransitContextResolver
type: feat
operator-signal:
scope: VFOmKssE5/VFOvGdksF
index: 2
started_at: 2026-03-30T18:31:00
submitted_at: 2026-03-30T18:32:33
completed_at: 2026-03-30T18:32:35
---

# Implement ContextResolver Port And TransitContextResolver

## Summary

Implement the `ContextResolver` port and its transit-backed implementation `TransitContextResolver`. This enables resolving `ContextLocator::Transit` variants to full artifact content using the transit trace recorder.

## Acceptance Criteria

- [x] ContextResolver port trait with async resolve(locator) -> Result<String> method [SRS-03/AC-01] <!-- verify: cargo test -- infrastructure::adapters::transit_resolver::tests, SRS-03:start:end, proof: tests_passed.log -->
- [x] TransitContextResolver implements ContextResolver using TransitTraceRecorder replay [SRS-04/AC-01] <!-- verify: cargo test -- infrastructure::adapters::transit_resolver::tests, SRS-04:start:end, proof: tests_passed.log -->
- [x] Resolution is lazy — only performed on explicit request [SRS-NFR-01/AC-01] <!-- verify: manual, SRS-NFR-01:start:end, proof: code_audit.log -->
