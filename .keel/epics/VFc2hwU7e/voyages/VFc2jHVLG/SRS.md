# Plan Inception Provider Delivery - SRS

## Summary

Epic: VFc2hwU7e
Goal: Deliver first-class Inception Labs provider support in staged slices:
core Mercury-2 chat compatibility first, then optional diffusion streaming and
edit-native capabilities.

## Scope

### In Scope

- [SCOPE-01] Add `Inception` as a first-class provider in the catalog, credential store, CLI/TUI login flow, and `/model` surfaces
- [SCOPE-02] Route `mercury-2` through the existing OpenAI-compatible HTTP adapter with structured output and forensic capture support
- [SCOPE-03] Document recommended Inception usage and operator-facing defaults once the provider is available
- [SCOPE-04] Plan a dedicated follow-on slice for streaming/diffusion visualization that builds on the core provider path
- [SCOPE-05] Plan a dedicated follow-on slice for edit-native endpoint support that remains distinct from chat completions

### Out of Scope

- [SCOPE-06] Upstream `sift` changes as a prerequisite for bringing up the Inception remote provider
- [SCOPE-07] Replacing the local-first runtime or changing the default provider/model during the initial provider-enablement slice
- [SCOPE-08] Shipping edit-native or streaming/diffusion capabilities inside the same implementation slice as basic Mercury-2 support

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must expose Inception as an authenticated provider in provider selection, credential handling, `/login`, and `/model` visibility surfaces. | SCOPE-01 | FR-01 | cargo test |
| SRS-02 | Paddles must run `mercury-2` through the existing OpenAI-compatible HTTP adapter, including structured answer normalization and forensic exchange artifacts. | SCOPE-02 | FR-02 | cargo test |
| SRS-03 | Paddles must document recommended Inception setup, supported model selection, and the split between core support and optional native capabilities. | SCOPE-03 | FR-03 | manual |
| SRS-04 | The plan must contain a discrete follow-on slice for streaming/diffusion visualization rather than folding it into the core provider slice. | SCOPE-04 | FR-04 | board |
| SRS-05 | The plan must contain a discrete follow-on slice for edit-native endpoint support separate from chat completions. | SCOPE-05 | FR-05 | board |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Missing Inception credentials must fail closed without degrading availability or behavior of existing providers. | SCOPE-01 | NFR-01 | cargo test |
| SRS-NFR-02 | Core Inception support must reuse the existing HTTP provider and rendering contracts instead of creating a one-off execution path. | SCOPE-02 | NFR-02 | cargo test |
| SRS-NFR-03 | Optional streaming/diffusion and edit-native work must remain explicitly non-blocking to the Mercury-2 compatibility slice. | SCOPE-04 | NFR-03 | board |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
