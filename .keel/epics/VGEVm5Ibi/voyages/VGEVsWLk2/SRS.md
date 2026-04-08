# Extract Runtime Shell And Chat Boundaries - SRS

## Summary

Epic: VGEVm5Ibi
Goal: Break the runtime shell into modular app, chat, and store boundaries without changing transcript, composer, or routing behavior.

## Scope

### In Scope

- [SCOPE-01] Extract app shell composition, transcript rendering, composer behavior, and manifold-turn-selection context into dedicated React modules.
- [SCOPE-02] Separate runtime bootstrap/SSE/send-turn transport and event reduction from shell layout code.
- [SCOPE-03] Preserve prompt history, multiline paste compression, sticky-tail chat scrolling, and transcript-driven manifold turn selection during extraction.

### Out of Scope

- [SCOPE-04] Decomposing inspector, manifold, or transit route internals beyond the shell interfaces they consume.
- [SCOPE-05] Reworking runtime styling beyond the minimal imports needed to support extracted modules.
- [SCOPE-06] Replacing the embedded fallback shell implementation.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | Define an explicit module map and migration contract for app, chat, and store extraction, including which shared state remains shell-owned and which moves into domain modules. | SCOPE-01 | FR-01 | docs + tests |
| SRS-02 | Extract transcript, message rendering, composer behavior, and manifold-turn-selection wiring into dedicated modules while preserving current behavior. | SCOPE-02 | FR-01 | tests |
| SRS-03 | Separate runtime bootstrap, projection stream, and event-log reduction concerns into dedicated store/client modules without breaking the existing shell-facing store API. | SCOPE-02 | FR-01 | tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Preserve chat/composer behavior, prompt history recall, sticky-tail scrolling, and transcript-driven manifold selection during the extraction. | SCOPE-03 | NFR-01 | tests |
| SRS-NFR-02 | Keep the new module seams explicit and low-coupling so later route extractions can build on them instead of reopening the shell file. | SCOPE-01 | NFR-02 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
