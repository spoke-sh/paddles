# Dogma and Bias Calibration - SRS

## Summary

Epic: VE5bGJZTR
Goal: Implement environmental biases and immutable religious dogma validation during boot.

## Scope

### In scope

- [SCOPE-01] Implementation of environmental biases (offset calibration).
- [SCOPE-02] Implementation of religious dogma validation (immutable invariants).

### Out of scope

- [SCOPE-03] Complex theological reasoning or dynamic dogma updates.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-08] | CLI must accept `--biases` argument | SCOPE-01 | FR-01 | manual |
| [SRS-09] | CLI must validate "Simulation over Reality" dogma | SCOPE-02 | FR-02 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| [SRS-NFR-04] | Errors must report "Unclean Boot" status | SCOPE-02 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
