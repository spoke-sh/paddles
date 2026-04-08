# Build Deterministic Resolver Backbone - Software Design Description

> Give the planner a deterministic, cache-backed way to resolve likely workspace entities into authored file targets before it spends edit budget on broad search or malformed patch attempts.

**SRS:** [SRS.md](SRS.md)

## Overview

Introduce a resolver seam that sits between raw user/planner hints and workspace-edit targeting. The resolver owns deterministic normalization, index lookup, candidate ranking, and explicit miss/ambiguity outcomes. It is self-discovering: it derives its inventory from the repository workspace rather than relying on IDE-fed state.

## Context & Boundaries

This voyage owns the resolver backbone only. It does not decide when the planner must invoke the resolver, and it does not yet change UI behavior beyond the artifacts needed for downstream integration.

```
┌──────────────────────────────────────────────┐
│ Resolver Port + Query Normalization         │
│        ↓                                    │
│ Authored Workspace Inventory + Cache        │
│        ↓                                    │
│ Deterministic Candidate Ranking / Misses    │
└──────────────────────────────────────────────┘
        ↑                     ↑
 WorkspacePathPolicy     Planner / Editor Clients
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `WorkspacePathPolicy` | internal | Reuse authored-workspace and `.gitignore` boundary logic | existing |
| Filesystem metadata | platform | Discover authored files and invalidate cache safely | std |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Resolver input model | Normalize free-form hints into deterministic lookup modes | Keeps ranking explainable and testable |
| Index source | Self-discover authored workspace files only | Avoids IDE coupling and keeps safety aligned with edit boundaries |
| Failure semantics | Return explicit miss/ambiguity results | Safer than silently guessing one candidate |

## Architecture

The backbone should likely add:
- A domain port for resolver queries and outcomes.
- An infrastructure adapter that inventories authored files and persists machine-managed cache state.
- A ranking layer that can combine exact path matches, basename matches, and symbol-like fragment matches deterministically.

## Components

- `EntityResolver` port: stable contract for planner/controller callers.
- `WorkspaceEntityIndex`: authored-file inventory plus cache invalidation metadata.
- `ResolverQueryNormalizer`: turns planner/user hints into exact-path, basename, and symbol-fragment lookups.
- `ResolverOutcome`: candidate list, ambiguity metadata, or miss diagnostics.

## Interfaces

Inputs should include the raw hint, optional likely-target context, and the workspace root. Outputs should carry:
- ranked authored file candidates
- confidence/ambiguity state
- a deterministic explanation artifact suitable for planner notes and runtime traces

## Data Flow

1. Caller submits an entity/path hint.
2. Resolver normalizes the hint into deterministic lookup modes.
3. Index/cache provides authored-file inventory.
4. Resolver ranks candidate files or returns miss/ambiguity.
5. Outcome becomes consumable by the planner/controller in voyage two.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Cache is stale | inventory metadata diverges from workspace state | rebuild affected index state | rerun lookup against rebuilt cache |
| Hint points outside authored boundary | `WorkspacePathPolicy` rejects candidate | return deterministic miss | planner keeps search inside authored files |
| Multiple candidates remain tied | ranking cannot break ambiguity safely | return ambiguity outcome with candidates | planner/controller asks for narrower evidence or chooses bounded follow-up |
