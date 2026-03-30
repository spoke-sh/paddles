# Interpretation Visibility - SRS

## Summary

Epic: VFNvFQPuA
Goal: Surface structured interpretation context in the TUI transcript with tiered verbosity

## Scope

### In Scope

- [SCOPE-01] Enrich TurnEvent::InterpretationContext payload with category counts
- [SCOPE-02] Tiered TUI rendering (default/v/vv) for interpretation events
- [SCOPE-03] New TurnEvent for guidance graph expansion

### Out of Scope

- [SCOPE-04] Modifying the interpretation derivation logic
- [SCOPE-05] Web UI rendering

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | TurnEvent::InterpretationContext carries doc_count, hint_count, procedure_count, and compact detail string | SCOPE-01 | FR-01 | manual |
| SRS-02 | Default TUI renders category breakdown line; -v adds source previews and hints; -vv shows full render | SCOPE-02 | FR-03 | manual |
| SRS-03 | New TurnEvent::GuidanceGraphExpanded with docs_discovered, depth_reached, root_sources | SCOPE-03 | FR-05 | manual |
| SRS-04 | GuidanceGraphExpanded emitted from sift_agent after graph expansion, min_verbosity=1 | SCOPE-03 | FR-05 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Rendering respects existing verbosity tier and pace promotion | SCOPE-02 | NFR-01 | manual |
| SRS-NFR-02 | No additional model calls for rendering | SCOPE-01 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
