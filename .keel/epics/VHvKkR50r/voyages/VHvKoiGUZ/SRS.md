# Remove Npm Audit Vulnerabilities - SRS

## Summary

Epic: VHvKkR50r
Goal: Upgrade or pin web UI dependency versions so npm audit reports zero vulnerabilities while preserving existing frontend and docs behavior.

## Scope

### In Scope

- [SCOPE-01] Root npm workspace dependency resolution and lockfile.
- [SCOPE-02] `@paddles/web` and `@paddles/docs` dependency manifests when direct version bumps are required.
- [SCOPE-03] Existing npm audit, lint, test, build, and e2e verification.

### Out of Scope

- [SCOPE-04] Rust dependency changes.
- [SCOPE-05] New web UI or docs functionality.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The npm workspace dependency graph reports zero vulnerabilities after the refresh. | SCOPE-01 | FR-01 | npm audit |
| SRS-02 | Existing web and docs quality gates continue to pass after dependency changes. | SCOPE-03 | NFR-01 | npm run lint && npm run test && npm run build && npm run e2e |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Dependency updates preserve the existing npm workspace layout and package manager contract. | SCOPE-01 | NFR-01 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
