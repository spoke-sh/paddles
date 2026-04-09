# Decouple Brain From Hands In The Local Runtime - Software Design Description

> Decouple the recursive brain from local execution hands so workspace tools, transports, and future runtimes can fail, recover, and swap independently.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage separates recursive reasoning from local execution environments. The harness should treat workspace edits, terminal commands, and transport/session endpoints as hands with a shared lifecycle rather than as hard-coded special cases.

## Context & Boundaries

In scope are local execution surfaces and the security/diagnostic boundaries around them. Out of scope are hosted sandboxes, IDE-fed state, and profile-driven controller behavior.

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

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| local workspace editor | internal boundary | Authored-file mutation and diff execution | existing adapter |
| background terminal runner | internal boundary | Shell execution as a typed hand | existing terminal adapter |
| native transport registry | internal boundary | Shared diagnostics for transport-facing hands | existing registry |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Hand abstraction | Use one shared lifecycle vocabulary for local execution surfaces | Recovery and observability should not depend on tool-specific wording |
| Security boundary | Mediate credentials outside generated-code execution paths | Structural isolation generalizes better than prompt rules |
| Diagnostics | Report hand state through existing trace and transport surfaces | Operators already know where to look |

## Architecture

The brain remains the recursive planner/controller loop. Hands become typed local execution adapters that can be provisioned, invoked, degraded, or recovered independently, while still feeding their state back into the same recorder and diagnostics layers.

## Components

- hand lifecycle contract
- workspace-editor hand adapter
- terminal-execution hand adapter
- transport/tool mediator layer for credential-bearing calls
- diagnostics projection layer for hand health and failures

## Interfaces

- hand operations: describe, provision, execute, recover, degrade
- mediator interfaces for privileged external actions
- trace/diagnostic emission for hand lifecycle transitions

## Data Flow

Planner/controller selects an action, the controller routes it to a hand, the hand reports execution and failure state through shared lifecycle events, and any privileged external interaction flows through a mediator that keeps secrets out of generated code.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Hand unavailable | lifecycle/provision check fails | surface degraded hand status and block unsafe execution | reprovision or switch hand |
| Credential mediation failure | transport/tool mediator rejects or cannot authorize | fail closed with explicit diagnostic | re-authenticate or retry through mediator |
| Workspace boundary violation | hand targets non-authored path or unsafe action | reject execution with structured error | redirect to valid authored hand target |
