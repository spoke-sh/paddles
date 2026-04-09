---
# system-managed
id: VGGIor3dC
created_at: 2026-04-08T20:45:35
# authored
title: Reframe Trace Surfaces As A Narrative Machine
index: 49
mission: VGGIo7WWc
---

# Reframe Trace Surfaces As A Narrative Machine

> The current transit trace inspector and forensic inspector expose raw trace structure instead of a coherent causal story, so operators see too many records, modes, and filters without a simple narrative of how a turn moved through planning, steering, tools, and outcome.

## Shared Contract

Later voyages in this epic must preserve one shared operator selection model across transit and forensic:

- selected turn
- selected moment
- optional internals

The default path is always narrative-first. Raw record ids, trace ids, payloads, and comparison details remain available through internals, but they are no longer the first surface the operator has to parse.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 3/3 voyages complete, 9/9 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Define Narrative Machine Model And Shared Projection](voyages/VGGIqsj2g/) | done | 3/3 |
| [Build Turn Machine Stage For Transit](voyages/VGGIqtM2e/) | done | 3/3 |
| [Simplify Forensic Inspector Around Machine Narrative](voyages/VGGIqts2y/) | done | 3/3 |
<!-- END GENERATED -->
