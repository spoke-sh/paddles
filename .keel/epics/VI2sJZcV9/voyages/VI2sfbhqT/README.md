---
# system-managed
id: VI2sfbhqT
status: in-progress
epic: VI2sJZcV9
created_at: 2026-04-27T18:37:56
# authored
title: Rename MechSuitService And Chambers To Idiomatic Modules
index: 1
updated_at: 2026-04-27T18:46:11
started_at: 2026-04-27T19:47:54
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
**Progress:** 2/4 stories complete

| Title | Type | Status |
|-------|------|--------|
| [Rename MechSuitService to AgentRuntime](../../../../stories/VI2slwMGg/README.md) | refactor | done |
| [Rename Chamber Wrappers To Plain Modules](../../../../stories/VI3BWyxR3/README.md) | refactor | icebox |
| [Rename Recursive Control Module To Agent Loop](../../../../stories/VI3BX17Sw/README.md) | refactor | icebox |
| [Trust Operator Memory Over Probe Procedure](../../../../stories/VI3fgYucO/README.md) | refactor | done |
<!-- END GENERATED -->
