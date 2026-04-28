# Rewrite Docs To Renamed Vocabulary And Drop Aspirational Sections - Software Design Description

> Rewrite README, ARCHITECTURE, CONSTITUTION, PROTOCOL, STAGE, POLICY, INSTRUCTIONS, CONFIGURATION, and AGENTS to use the renamed vocabulary; remove or move-to-roadmap sections that describe non-existent capability (automatic tier promotion, concurrent sibling generation, deterministic entity resolution as deterministic, specialist brains beyond session-continuity-v1).

**SRS:** [SRS.md](SRS.md)

## Overview

The top-level docs (`README.md`, `ARCHITECTURE.md`, `CONSTITUTION.md`, `PROTOCOL.md`, `STAGE.md`, `POLICY.md`, `INSTRUCTIONS.md`, `CONFIGURATION.md`, `AGENTS.md`) currently use bespoke vocabulary that the Tier-1 rename voyages are retiring (`MechSuitService`, `*Chamber`, `ExecutionHand`, `WorkspaceAction`, `specialist_brains`, `harness_profile`, `gatherer`, `forensics`, `compaction_cue`, `premise_challenge`, `deliberation_signals`, `steering_signals`) and they describe capability the codebase does not ship today (automatic tier promotion, concurrent sibling generation, deterministic entity resolution framed as deterministic, specialist brains beyond `session-continuity-v1`). The result is documentation that overpromises and uses terms a new contributor cannot grep for.

This voyage rewrites the docs to use the renamed vocabulary (`AgentRuntime`, `agent_loop`, `Tool`, `ToolExecutor`, `subagents`, `runtime_profile`, `retriever`, `trace`/`inspector`, `controller_signals`, `reasoning_signals`, `evidence_check`, `compaction_trigger`) and honestly distinguishes shipped capability from roadmap aspirations.

The first slice (this story) covers `README.md` and `ARCHITECTURE.md` end-to-end. Subsequent stories under this voyage sweep the rest. Every rewrite is bounded by three rules:

1. **Use renamed terms.** Replace every retired bespoke term with its industry-standard alias.
2. **Truthful about capability.** Sections describing unshipped capability are deleted, demoted to a clearly marked "Roadmap" subsection, or moved into a dedicated `ROADMAP.md`. No shipped section may continue to claim capability the code does not provide.
3. **No new bespoke vocabulary.** New concepts adopt established agent-tooling terms (ReAct, tool use, plan mode, subagent, MCP server, hook, slash command, system prompt, retriever).

The rewrite is prose-only. No `apps/docs` restructure, no docs build pipeline change, no new tutorials.

## Components

- `README.md` — rewrite "Backbone Architecture", "Steering Signals", "Web Trace Routes", and any other sections containing retired terms; remove or move-to-roadmap unshipped capability claims.
- `ARCHITECTURE.md` — same treatment, particular attention to the manifold / steering / chamber framing and explicit "future work" notes.
- Tier-2 doc sweep (subsequent stories): `CONSTITUTION.md`, `PROTOCOL.md`, `STAGE.md`, `POLICY.md`, `INSTRUCTIONS.md`, `CONFIGURATION.md`, `AGENTS.md`.
- Optional new file: `ROADMAP.md` — single home for moved aspirational sections, clearly labeled.
- Verification: `git grep -F` for each retired term across tracked top-level docs returns no hits; manual review confirms every shipped claim is backed by code in `src/`.

## Context & Boundaries

<!-- What's in scope, what's out of scope, external actors/systems we interact with -->

```
┌─────────────────────────────────────────┐
│              This Voyage                │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │         │  │         │  │         │ │
│  └─────────┘  └─────────┘  └─────────┘ │
└─────────────────────────────────────────┘
        ↑               ↑
   [External]      [External]
```

## Dependencies

<!-- External systems, libraries, services this design relies on -->

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|

## Architecture

<!-- Component relationships, layers, modules -->

## Components

<!-- For each major component: purpose, interface, behavior -->

## Interfaces

<!-- API contracts, message formats, protocols (if this voyage exposes/consumes APIs) -->

## Data Flow

<!-- How data moves through the system; sequence diagrams if helpful -->

## Error Handling

<!-- What can go wrong, how we detect it, how we recover -->

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
