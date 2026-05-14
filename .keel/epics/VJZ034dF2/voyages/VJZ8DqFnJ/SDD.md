# Remove In-Process Sift Inference Code - Software Design Description

> Delete paddles-owned Sift model loading and in-process inference adapters after HTTP-only runtime construction is proven, without removing Sift retrieval/indexing.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes the dead in-process Sift inference implementation after the
runtime has been moved to HTTP model clients. The goal is to delete the code and
dependencies that made paddles responsible for model loading, while preserving
Sift retrieval/indexing for a separate later decision.

## Context & Boundaries

The previous voyages should have already made Sift model-provider configuration
fail before runtime construction. That means Sift inference adapters should no
longer be reachable. This voyage removes them and any inference-only build
surface, but it stops if retrieval depends on code that would otherwise be
deleted.

```
┌─────────────────────────────────────────┐
│       Sift Inference Deletion           │
│                                         │
│  delete inference adapters              │
│  remove model prep and deps             │
│  keep retrieval/indexing                │
└─────────────────────────────────────────┘
        ↑               ↑
 HTTP runtime     retrieval tests
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| HTTP-only runtime construction | prior voyage | Ensures Sift inference is unreachable before deletion | repo-local |
| Cargo/build files | build config | Remove unused inference-only dependencies | repo-local |
| Sift retrieval tests | Rust tests | Prove retrieval remains independent | repo-local |
| Docs | project docs | Remove local model loading instructions | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Delete inference code | Yes | Leaving dead Sift inference branches would undermine the cleanup. |
| Remove dependencies | When unused | Build surface should reflect runtime architecture. |
| Retrieval | Preserve | Retrieval/indexing remains a separate product and architecture decision. |
| Docs | Purge model-loading guidance | Operators should not be taught unsupported local loading paths. |

## Architecture

After this voyage, all inference flows go through HTTP model clients. Any Sift
module that remains must be retrieval/indexing-specific or compatibility parsing
that fails before runtime construction.

## Components

| Component | Purpose | Behavior |
|-----------|---------|----------|
| Deleted Sift inference adapters | Former local inference implementation | Removed from active source tree |
| Provider compatibility parser | Legacy detection | Retains enough parsing to return migration errors |
| Retrieval adapters | Context/indexing behavior | Remain selectable and tested |
| Build/dependency config | Dependency surface | Drops unused inference-only crates |

## Interfaces

No new runtime API is introduced. Existing legacy Sift model-provider inputs
continue to fail with the migration hint established by the ADR voyage.

## Data Flow

1. Compile/test failures expose references to Sift inference adapters.
2. Runtime factories and registries are simplified around HTTP model clients.
3. Unused inference-only dependencies are removed.
4. Retrieval tests prove Sift retrieval/indexing remains independent.
5. Docs are updated to remove local model loading guidance.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Retrieval imports inference-only code | Compile or retrieval tests | Stop deletion and split retrieval refactor | Keep retrieval-specific code or plan a new mission |
| Legacy Sift provider reaches runtime | Compatibility tests | Fail the story | Restore early hard-fail path |
| Dependency removal breaks HTTP providers | Full test suite | Restore only required dependency or split refactor | Keep HTTP provider coverage green |
