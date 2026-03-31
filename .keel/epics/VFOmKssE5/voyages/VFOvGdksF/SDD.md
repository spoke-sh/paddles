# Context Locator And Transit Resolution - Software Design Description

> Define ContextLocator type and implement transit read-back resolution

**SRS:** [SRS.md](SRS.md)

## Overview

Introduce a `ContextLocator` enum in the domain model that addresses context artifacts across four tiers. Implement a `ContextResolver` port trait with a transit-backed adapter that reads specific records from transit streams. Wire the resolver into `PlannerLoopContext` so the planner can resolve truncated artifact locators on demand.

## Context & Boundaries

```
┌──────────────────────────────────────────────────────────────┐
│                  domain/model (new types)                     │
│  ┌──────────────┐  ┌────────────────┐                        │
│  │ContextLocator│  │ContextResolver │ (port trait)           │
│  └──────────────┘  └────────────────┘                        │
└──────────────────────────────────────────────────────────────┘
         ↑                       ↑
┌────────┴──────────┐   ┌───────┴──────────────────────┐
│  ArtifactEnvelope │   │ TransitContextResolver       │
│  (updated locator)│   │ (infrastructure adapter)     │
└───────────────────┘   │ uses TransitTraceRecorder    │
                        └──────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| ArtifactEnvelope | Internal | Existing truncation + locator mechanism | paddles-conversation |
| TransitTraceRecorder | Internal | Transit engine access for read-back | infrastructure/adapters |
| PlannerLoopContext | Internal | Carries resolver into planner loop | application/mod.rs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Locator is domain type, not infra | ContextLocator lives in domain/model | Preserves domain boundary — no transit-core types leak through |
| Resolver is a port trait | ContextResolver trait in domain/ports | Multiple backends possible (transit, in-memory for tests) |
| Transit resolution by replay + filter | Read all records for task, filter by record_id | Leverages existing replay() API without new transit-core changes |

## Components

### ContextLocator (domain/model/context.rs)

```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ContextLocator {
    Inline { content: String },
    Transit { task_id: String, record_id: String },
    Sift { index_ref: String },
    Filesystem { path: String },
}
```

### ContextResolver (domain/ports/context_resolution.rs)

```rust
#[async_trait]
pub trait ContextResolver: Send + Sync {
    async fn resolve(&self, locator: &ContextLocator) -> Result<String>;
}
```

### TransitContextResolver (infrastructure/adapters/transit_resolver.rs)

Wraps `TransitTraceRecorder`, implements `ContextResolver` for `Transit` variant locators. Uses `recorder.replay(task_id)` and filters records by `record_id` to extract artifact content.

## Data Flow

1. During context assembly, artifact exceeds inline limit → `ArtifactEnvelope` truncates and generates `ContextLocator::Transit { task_id, record_id }`
2. Record is written to transit stream (already happening today)
3. Later, planner encounters truncated artifact → calls `resolver.resolve(&locator)`
4. `TransitContextResolver` replays task stream, finds matching record, returns full content

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Transit stream missing for task_id | replay() returns empty | Return error with context | Use truncated inline content as fallback |
| Record not found in stream | Filter returns no match | Return error with record_id | Use truncated inline content |
| Locator variant not supported by this resolver | Match on non-Transit variant | Return unsupported error | Caller routes to appropriate resolver |
