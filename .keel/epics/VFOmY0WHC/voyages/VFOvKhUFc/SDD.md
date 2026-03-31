# Tier Model And Cross-Tier Locator Resolution - Software Design Description

> Formalize four-tier context model and implement cross-tier locator resolution

**SRS:** [SRS.md](SRS.md)

## Overview

Formalize the implicit four-tier context model (inline, transit, sift, filesystem) as an explicit `ContextTier` enum. Extend `ArtifactEnvelope` to carry typed `ContextLocator` with tier metadata. Implement cross-tier resolution for inline->transit and transit->filesystem paths. Resolution fails closed with explicit errors when tiers are unavailable.

## Context & Boundaries

```
┌──────────────────────────────────────────────────────────────┐
│  Context Tiers                                                │
│                                                               │
│  Tier 1: Inline     ─── truncation boundary ──→               │
│  Tier 2: Transit    ─── full records ──→                      │
│  Tier 3: Sift       ─── indexed evidence ──→                  │
│  Tier 4: Filesystem ─── workspace files ──→                   │
│                                                               │
│  Resolution paths:                                            │
│  Inline ──locator──→ Transit ──file ref──→ Filesystem         │
│                                                               │
│  ┌──────────────────┐                                         │
│  │ ContextResolver   │ dispatches to tier-specific resolvers   │
│  │ (chain of resp.)  │                                        │
│  └──────────────────┘                                         │
└──────────────────────────────────────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| ContextLocator | Internal | Addressing type (from VFOmKssE5) | domain/model/context.rs |
| ContextResolver | Internal | Resolution trait (from VFOmKssE5) | domain/ports/context_resolution.rs |
| ArtifactEnvelope | Internal | Existing truncation mechanism | paddles-conversation |
| TransitTraceRecorder | Internal | Transit read-back | infrastructure/adapters |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Tier as compile-time enum | ContextTier enum, not runtime registry | Type safety, exhaustive matching |
| Chain-of-responsibility resolution | Resolver tries tiers in order: inline, transit, filesystem | Local-first, honest degradation |
| Fail-closed on missing tier | Return explicit error, never silently degrade | Per charter: fail-closed, never panic or block |

## Components

### ContextTier (domain/model/context.rs)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContextTier {
    Inline,    // < configurable char limit, truncated with locator
    Transit,   // full records in transit streams, persistent
    Sift,      // indexed evidence via autonomous retrieval
    Filesystem,// workspace files, direct read access
}
```

### Extended ArtifactEnvelope (paddles-conversation)

Replace the bare `locator: Option<String>` with `locator: Option<ContextLocator>`. When an artifact is truncated, the locator carries both the tier and the resolution address.

### ChainedContextResolver (infrastructure/adapters/context_resolver.rs)

Dispatches resolution based on the locator's tier field:
- `Transit` → `TransitContextResolver`
- `Filesystem` → reads file from workspace root
- `Inline` → returns inline content directly
- `Sift` → returns unsupported error (future)

## Data Flow

1. Artifact truncated at inline tier → `ArtifactEnvelope` gets `ContextLocator::Transit { tier: Transit, task_id, record_id }`
2. Consumer calls `resolver.resolve(&locator)`
3. `ChainedContextResolver` dispatches to `TransitContextResolver` based on tier
4. Transit resolver replays stream, extracts full content
5. If transit record references a file path, consumer can request filesystem resolution

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Tier unavailable (e.g., transit engine not initialized) | Resolver returns TierUnavailable error | Explicit error message naming the tier | Consumer uses truncated inline as fallback |
| Record not found at target tier | Resolver returns RecordNotFound error | Error includes locator details for debugging | Consumer degrades honestly |
| Unsupported tier (e.g., Sift) | Match on tier variant | Return UnsupportedTier error | Consumer knows to not retry |
