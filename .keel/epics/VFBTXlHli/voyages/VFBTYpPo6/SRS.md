# Subagent Interface and Routing Foundations - SRS

## Summary

Epic: VFBTXlHli
Goal: Define a proper context-gathering subagent interface, evidence contract, and routing boundary so Paddles can later use Chroma Context-1 without replacing the default local answer runtime.

## Scope

### In Scope

- [SCOPE-01] Define a typed context-gathering subagent contract that accepts retrieval work and returns a ranked evidence bundle plus capability metadata.
- [SCOPE-02] Split runtime/model preparation so Paddles can represent distinct gatherer and synthesizer lanes.
- [SCOPE-03] Add routing for retrieval-heavy requests that runs context gathering before final answer synthesis.
- [SCOPE-04] Preserve the current local answer/tool path for turns that do not require context gathering or when the gatherer lane is unavailable.
- [SCOPE-05] Introduce an experimental Context-1 adapter boundary with explicit harness/capability gating plus operator-facing docs/debug visibility for the new lane.

### Out of Scope

- [SCOPE-06] Replacing the default answer runtime with Context-1.
- [SCOPE-07] Reproducing Chroma's non-public/private harness behavior inside Paddles.
- [SCOPE-08] Silent remote fallback for common prompt execution.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Paddles must define a typed context-gathering request/result interface that can return ranked evidence items, a synthesis-ready summary, and capability metadata for the active gatherer. | SCOPE-01 | FR-01 | manual |
| SRS-02 | Runtime and configuration wiring must support separate gatherer and synthesizer lanes instead of assuming one active answer model path. | SCOPE-02 | FR-02 | manual |
| SRS-03 | Retrieval-heavy requests must be classified and routed through the context-gathering lane before final answer synthesis. | SCOPE-03 | FR-03 | manual |
| SRS-04 | When a request is not retrieval-heavy, or the gatherer lane is unavailable, Paddles must preserve the existing default answer/tool path. | SCOPE-04 | FR-04 | manual |
| SRS-05 | An experimental Context-1 adapter boundary must expose explicit `available`, `unsupported`, or `harness-required` capability states. | SCOPE-05 | FR-05 | manual |
| SRS-06 | Verbose/debug output must report routing decisions and concise evidence bundle summaries so operators can inspect gatherer behavior. | SCOPE-05 | FR-03 | manual |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | The default answer/tool path must remain local-first and continue to operate without a mandatory new network dependency for common prompt handling. | SCOPE-02, SCOPE-04 | NFR-01 | manual |
| SRS-NFR-02 | Routing logic and gatherer integration must degrade safely and preserve deterministic workspace actions when the gatherer lane fails or is unsupported. | SCOPE-03, SCOPE-04, SCOPE-05 | NFR-03 | manual |
| SRS-NFR-03 | Unsupported specialized gatherer providers must fail closed with explicit operator-visible messaging rather than silently changing runtimes. | SCOPE-05 | NFR-03 | manual |
| SRS-NFR-04 | Routing and evidence behavior must be observable enough to debug misclassification and missing-context cases from verbose/debug output. | SCOPE-05 | NFR-02 | manual |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
