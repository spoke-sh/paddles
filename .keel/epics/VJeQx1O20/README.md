---
# system-managed
id: VJeQx1O20
created_at: 2026-05-14T19:15:03
# authored
title: Agent Loop Owns Turn Action Selection
index: 73
mission: VJeQmJ0Ro
---

# Agent Loop Owns Turn Action Selection

> Paddles still has a model-selected initial action and controller bootstrap path before the recursive agent loop runs. That split lets turn-mode policy, routing, grounding, edit, and commit pressure steer the request outside the loop, which can make simple questions stall or repeat pre-loop evidence actions instead of letting the agent loop reason over live observations.

## Documents

| Document | Description |
|----------|-------------|
| [PRD.md](PRD.md) | Product requirements and success criteria |
| `PRESS_RELEASE.md` (optional) | Working-backwards artifact for large user-facing launches; usually skip for incremental/refactor/architecture-only work |

## Voyages

<!-- BEGIN GENERATED -->
**Progress:** 0/3 voyages complete, 1/7 stories done
| Voyage | Status | Stories |
|--------|--------|---------|
| [Unify First Action Entry Point](voyages/VJeRAOoHj/) | in-progress | 1/2 |
| [Move Turn Contract Into Agent Loop](voyages/VJeRAPzHh/) | planned | 0/2 |
| [Retire Pre-Loop Bootstraps And Vocabulary](voyages/VJeRAR1IS/) | planned | 0/3 |
<!-- END GENERATED -->
