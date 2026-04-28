---
# system-managed
id: VI2sfbhqT
status: planned
epic: VI2sJZcV9
created_at: 2026-04-27T18:37:56
# authored
title: Rename MechSuitService And Chambers To Idiomatic Modules
index: 1
updated_at: 2026-04-27T18:46:11
---

# Rename MechSuitService And Chambers To Idiomatic Modules

> Mechanical, behavior-preserving renames: MechSuitService -> AgentRuntime; *Chamber -> plain function modules (agent_loop, context_assembly, synthesis, turn); RecursiveControlChamber -> agent_loop. Land each rename as its own reviewable diff. Subsequent voyages handle ExecutionHand, WorkspaceAction, specialist_brains, harness_profile, gatherer, forensics, and the steering/deliberation/compaction/premise term sweeps.

## Documents

<!-- BEGIN DOCUMENTS -->
| Document | Description |
|----------|-------------|
| [SRS.md](SRS.md) | Requirements and verification criteria |
| [SDD.md](SDD.md) | Architecture and implementation details |
<!-- END DOCUMENTS -->

## Stories

<!-- BEGIN GENERATED -->
**Progress:** 0/1 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Rename MechSuitService to AgentRuntime](../../../../stories/VI2slwMGg/README.md) | refactor | backlog |
<!-- END GENERATED -->
