# Split Application Into Hexagonal Boundaries - Software Design Description

> Reduce the application monolith by extracting domain-driven ports, application services, and infrastructure adapters without changing recursive behavior.

**SRS:** [SRS.md](SRS.md)

## Overview

Move from a large application module toward explicit ports and services. The domain keeps recursive vocabulary and invariants. Application services coordinate use cases. Infrastructure adapters handle providers, filesystem, transport, persistence, and UI integration.

## Context & Boundaries

The refactor is behavior-preserving. Every extraction starts with tests or snapshots around the existing behavior, then moves code into a smaller module while keeping public orchestration contracts stable.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing application module | application | Source for extraction | internal |
| Domain ports and models | domain | Target contracts | internal |
| Eval and test harness | development | Regression protection | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Refactor order | contract builder, planner loop, evidence, external service | Extract low-risk seams before changing features. |
| Domain rule | no infrastructure dependencies in domain | Keeps hexagonal boundary clean. |
| Verification | snapshot plus unit tests | Protects recursive behavior during movement. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| PlannerLoopService | application | Recursive action loop orchestration. |
| ExecutionContractService | application | Capability and constraint disclosure. |
| EvidenceService | application | Evidence bundle normalization. |
| Infrastructure adapters | infrastructure | Providers, workspace, transport, persistence. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Behavior drift | tests/snapshots | Stop refactor slice | Restore behavior before continuing. |
| Dependency violation | boundary check | Fail test | Move type or add port. |
| Extraction too broad | review | Split into smaller story | Preserve sealed slices. |
