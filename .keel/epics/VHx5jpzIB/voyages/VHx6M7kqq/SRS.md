# Expose Current OpenAI Pro Models - SRS

## Summary

Epic: VHx5jpzIB
Goal: Operators can select current OpenAI GPT-5.5 and pro model ids from the supported model catalog with correct transport capabilities.

## Scope

### In Scope

- [SCOPE-01] Add `gpt-5.5` and current OpenAI text/reasoning `*-pro` model IDs to the OpenAI provider catalog.
- [SCOPE-02] Preserve correct thinking-mode validation and runtime model ID preparation for GPT-5.5 family models.
- [SCOPE-03] Preserve Responses-oriented prompt-envelope capability negotiation for OpenAI pro models that require or use Responses continuity, and update owning configuration documentation when representative provider capability output changes.

### Out of Scope

- [SCOPE-04] Adding non-chat media generation models to the coding harness catalog.
- [SCOPE-05] Changing default lane model selections or reworking the OpenAI HTTP adapter beyond catalog and capability-surface classification.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The OpenAI provider catalog exposes `gpt-5.5`, `gpt-5.5-pro`, `gpt-5.4-pro`, `gpt-5.2-pro`, `gpt-5-pro`, `o3-pro`, and `o1-pro`. | SCOPE-01 | FR-01 | test |
| SRS-02 | `gpt-5.5` exposes the documented selectable reasoning efforts `none`, `low`, `medium`, `high`, and `xhigh`, and runtime IDs preserve selected thinking effort. | SCOPE-02 | FR-02 | test |
| SRS-03 | OpenAI pro model paths that use Responses continuity are classified as supported prompt-envelope planner/render paths with native continuation state. | SCOPE-03 | FR-02 | test |
| SRS-04 | Configuration documentation embeds the generated provider capability matrix after the OpenAI representative model paths are updated. | SCOPE-03 | NFR-02 | test |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Provider catalog and capability changes remain centralized in `src/infrastructure/providers.rs`. | SCOPE-01, SCOPE-03 | NFR-01 | review |
| SRS-NFR-02 | The change introduces no new runtime network dependency or non-local execution path. | SCOPE-01, SCOPE-03 | NFR-01 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
