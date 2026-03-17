# Interactive Loop Integration - SRS

## Summary

Epic: VE5oA4s7x
Goal: Implement the interactive prompt loop in main.rs.

## Scope

### In scope

- [SCOPE-01] Basic interactive loop in `main.rs` using `stdin`.
- [SCOPE-02] Shared session state across multiple prompts.
- [SCOPE-03] Integration with existing `CandleProvider`.

### Out of scope

- [SCOPE-04] Full TUI library integration (e.g. `ratatui`).

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-14 | Launch loop on empty prompt | SCOPE-01 | FR-01 | manual |
| SRS-15 | Persistent session across turns | SCOPE-02 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-07 | Display input prompt indicator | SCOPE-01 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
