# Establish A Typed External Capability Fabric Substrate - SRS

## Summary

Epic: VGb1c1XAL
Goal: Establish one typed external capability fabric for web, MCP, and connector-backed actions with evidence-first normalization and governed degradation.

## Scope

### In Scope

- [SCOPE-01] Define one capability contract for web search, MCP-backed tools, and connector-backed app actions.
- [SCOPE-02] Add planner and runtime discovery plus invocation semantics for those external capability classes.
- [SCOPE-03] Convert external results into evidence items, source records, and trace or runtime artifacts.
- [SCOPE-04] Integrate auth, approval, and unavailability handling with the broader execution-governance model.
- [SCOPE-05] Update docs and operator-visible diagnostics around external tool use.

### Out of Scope

- [SCOPE-06] A generic plugin marketplace or arbitrary third-party extension ecosystem.
- [SCOPE-07] Hosted connector backplanes or cloud-only auth infrastructure that would break local-first operation.
- [SCOPE-08] Review-mode or multi-agent semantics except where needed to consume the external capability fabric.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define a typed external capability descriptor and invocation contract for web, MCP, and connector-backed actions, including availability, auth posture, side-effect posture, and evidence shape. | SCOPE-01, SCOPE-04 | FR-01 | manual |
| SRS-02 | The planner and runtime must discover, select, and invoke external capabilities through the same recursive action loop used for local workspace and shell work. | SCOPE-01, SCOPE-02 | FR-02 | manual |
| SRS-03 | External results must normalize into evidence items, source records, and runtime artifacts with lineage, summaries, and availability state instead of remaining opaque tool output. | SCOPE-03 | FR-03 | manual |
| SRS-04 | External capability invocation must compose with auth, approval, and sandbox governance rather than bypassing the local execution policy model. | SCOPE-02, SCOPE-04 | FR-04 | manual |
| SRS-05 | Tool absence, auth failure, or stale capability metadata must degrade honestly with explicit runtime state and no false success. | SCOPE-02, SCOPE-04 | FR-05 | manual |
| SRS-06 | TUI, web, and API projections plus docs must expose active external fabrics, result provenance, and degraded states using one shared vocabulary. | SCOPE-03, SCOPE-05 | FR-06 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The recursive harness must preserve useful local-first operation when all external capability fabrics are absent or disabled. | SCOPE-02, SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | External capability metadata and results must remain observable through trace, transcript, and API surfaces. | SCOPE-03, SCOPE-05 | NFR-02 | manual |
| SRS-NFR-03 | External calls must respect the same policy and credential-boundary constraints as local execution hands. | SCOPE-04 | NFR-03 | manual |
| SRS-NFR-04 | Capability descriptors must remain generic enough to absorb new fabrics without reworking the recursive planner contract. | SCOPE-01, SCOPE-02 | NFR-04 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
