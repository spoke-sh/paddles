# Activate Recursive Delegation Runtime - Software Design Description

> Turn delegation models into bounded runtime workers that inherit governance, operate within explicit ownership, and return evidence to the parent recursive loop.

**SRS:** [SRS.md](SRS.md)

## Overview

Build a worker runtime around the existing delegation domain model. The parent turn owns spawning, context boundaries, evidence acceptance, and integration. Workers run through the same capability disclosure and governance contracts as the parent.

## Context & Boundaries

Domain owns worker request, ownership, lifecycle, and integration vocabulary. Application services own spawn, budget, context assembly, and evidence merge. Infrastructure may provide local execution handles, but workers do not bypass existing hands.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Delegation domain model | domain | Worker lifecycle and ownership vocabulary | internal |
| Recursive planner loop | application | Worker execution substrate | internal |
| Trace recorder | application/infrastructure | Worker projection and replay | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Parent ownership | parent integrates all worker output | Prevents unmanaged parallel changes. |
| Initial worker type | bounded local worker | Keeps scope testable. |
| Evidence contract | worker result becomes parent evidence | Preserves recursive traceability. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| WorkerRuntimePort | domain/application boundary | Spawn and observe bounded workers. |
| DelegationApplicationService | application | Context inheritance and evidence integration. |
| LocalWorkerAdapter | infrastructure | Executes local worker turns. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Worker exceeds budget | budget monitor | Stop worker, return partial evidence | Parent replans. |
| Ownership conflict | integration check | Reject artifact integration | Parent assigns new slice. |
| Worker failure | lifecycle status | Return failed evidence | Parent chooses fallback. |
