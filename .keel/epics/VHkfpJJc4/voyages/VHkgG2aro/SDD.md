# Implement Governed External Capability Broker - Software Design Description

> Replace the noop external capability broker with local-first governed web, MCP, and connector capability execution that returns typed evidence to the recursive planner.

**SRS:** [SRS.md](SRS.md)

## Overview

Introduce an application service that coordinates external capability calls through a domain port. Infrastructure adapters implement web, MCP, or connector-specific execution. The recursive planner sees only the typed capability contract, governance posture, and evidence results.

## Context & Boundaries

Domain owns capability descriptors, request/result types, effect metadata, and evidence shape. Application orchestration owns broker selection, governance checks, timeout policy, and evidence attachment. Infrastructure owns provider clients, auth, serialization, and network behavior.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing external capability domain model | domain | Capability descriptor and result contract | internal |
| Execution governance | application/domain | Permission and posture checks | internal |
| Optional web/MCP/connectors | infrastructure | Concrete capability implementations | opt-in |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Default posture | unavailable unless configured | Preserves local-first operation. |
| Architecture | broker behind a domain port | Keeps external tools out of core recursion. |
| Evidence | typed result for every outcome | Lets recursion reason over failures and degraded tools. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| ExternalCapabilityBroker port | domain/application boundary | Declares catalog and invokes capabilities. |
| ExternalCapabilityService | application | Applies governance, timeout, trace, and evidence mapping. |
| Web/MCP/Connector adapters | infrastructure | Execute concrete capability calls. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Capability unavailable | catalog status | Return unavailable result | Planner can choose local-only path. |
| Permission denied | governance decision | Return denied evidence | Planner can stop, explain, or request different path. |
| Adapter failure | timeout/error | Return degraded result | Evidence records provenance and retry posture. |
