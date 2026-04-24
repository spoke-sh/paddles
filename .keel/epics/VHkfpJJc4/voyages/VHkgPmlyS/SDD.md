# Expose Operator Surfaces And Provider Registry - Software Design Description

> Expose new capability posture, governance prompts, provenance, diagnostics, worker evidence, eval results, and provider/model registry behavior through operator-facing runtime surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

Extend existing CLI, TUI, web, and runtime projection contracts so operators can see the capabilities and constraints the recursive planner sees. Provider/model registry work stays behind a domain/application boundary and treats discovery as optional.

## Context & Boundaries

Domain owns provider and capability posture states. Application services project runtime facts. Infrastructure adapters render CLI/TUI/web views and load provider data.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Turn events and projections | application/domain | Runtime event source | internal |
| Provider registry/config | infrastructure | Existing provider posture | internal |
| CLI/TUI/web surfaces | infrastructure | Operator presentation | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Surface strategy | extend existing projections | Avoids a UI rewrite. |
| Provider posture | configured/discovered/unavailable/deprecated states | Makes model availability explicit. |
| Docs | local-first first | Keeps network capability setup honest. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| RuntimePostureProjection | application | Normalize capability, policy, provider, and eval posture. |
| ProviderRegistryPort | domain/application boundary | Model availability and metadata contract. |
| CLI/TUI/Web adapters | infrastructure | Present runtime posture to operators. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Provider discovery fails | adapter error | Mark unavailable with reason | Operator configures or ignores. |
| Projection event missing | projection check | Omit field with warning | Add event coverage in source slice. |
| Packaging smoke fails | test failure | Block release slice | Fix entrypoint or docs. |
