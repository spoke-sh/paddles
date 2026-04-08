# Split Route Surfaces Into Domain Modules - SRS

## Summary

Epic: VGEVm5Ibi
Goal: Move inspector, manifold, and transit into dedicated domain modules with localized state and view composition.

## Scope

### In Scope

- [SCOPE-01] Extract inspector, manifold, and transit into dedicated route modules with local hooks, selectors, and presentation components.
- [SCOPE-02] Move route-specific derived state and geometry helpers next to the route that owns them.
- [SCOPE-03] Preserve current route interactions, selectors, IDs, and regression coverage while splitting the code.

### Out of Scope

- [SCOPE-04] Reworking shared shell/chat/store boundaries beyond the interfaces established by voyage one.
- [SCOPE-05] Styling partition and fallback-shell contract work beyond minimal adjustments required to keep the routes compiling.

## Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-01 | The inspector route must compose dedicated modules for overview, navigation, records, detail panes, and selection helpers instead of one monolithic route body. | SCOPE-01 | FR-02 | tests |
| SRS-02 | The manifold route must compose dedicated modules/hooks for playback, camera interaction, gate-field construction, and readout presentation. | SCOPE-01 | FR-02 | tests |
| SRS-03 | The transit route must compose dedicated modules/hooks for toolbar state, board layout, panning/zoom, and trace node rendering. | SCOPE-01 | FR-02 | tests |
<!-- END FUNCTIONAL_REQUIREMENTS -->

## Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Scope | Source | Verification |
|----|-------------|-------|--------|--------------|
| SRS-NFR-01 | Preserve existing route behavior, DOM hooks, and interaction contracts while route code is split into domain modules. | SCOPE-03 | NFR-01 | tests |
| SRS-NFR-02 | Keep route-local helper code co-located with the owning route so later maintenance does not recreate another global helper monolith. | SCOPE-02 | NFR-02 | review |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->
