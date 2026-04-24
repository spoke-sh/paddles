# Expose Operator Surfaces And Provider Registry - SRS

## Summary

Epic: VHkfpJJc4
Goal: Expose new capability posture, governance prompts, provenance, diagnostics, worker evidence, eval results, and provider/model registry behavior through operator-facing runtime surfaces.

## Scope

### In Scope

- [SCOPE-09] Improve product/runtime surfaces only where needed to expose the new governance, eval, edit, diagnostic, external provenance, and worker evidence paths, including provider/model registry posture.

### Out of Scope

- [SCOPE-12] Full UI redesign.
- [SCOPE-11] Mandatory hosted service mode.
- [SCOPE-12] Replacing existing provider adapters in one slice.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Surface external capability, policy, edit diagnostic, LSP, delegation, and eval outcomes in existing projections. | SCOPE-09 | FR-09 | test: projection event snapshots |
| SRS-02 | Add provider/model registry posture for configured, discovered, unavailable, and deprecated model entries. | SCOPE-09 | FR-09 | test: registry posture tests |
| SRS-03 | Publish operator docs for configuring capabilities, running evals, and preserving local-first boundaries. | SCOPE-09 | FR-09 | manual: docs review |
| SRS-04 | Verify packaging/runtime entrypoints expose the new harness capabilities consistently. | SCOPE-09 | FR-09 | test: CLI/runtime smoke tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Surfaces expose runtime reality without synthetic controller plans. | SCOPE-09 | NFR-03 | test: projection content tests |
| SRS-NFR-02 | Provider discovery never forces network use in default local-first mode. | SCOPE-09 | NFR-01 | test: offline provider posture |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
