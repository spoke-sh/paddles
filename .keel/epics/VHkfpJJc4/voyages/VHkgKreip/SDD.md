# Install Codex-Grade Execution Policy - Software Design Description

> Add an expressive command and tool policy engine beneath Paddles governance so shell, edit, patch, and external actions can be allowed, prompted, denied, or retried with typed evidence.

**SRS:** [SRS.md](SRS.md)

## Overview

Add a deterministic policy evaluator below the existing governance gate. The domain model describes rules and decisions; application services call the evaluator before execution; infrastructure adapters provide executable metadata and environment context.

## Context & Boundaries

The recursive planner receives policy posture and outcomes, but never decides whether a restricted action is safe. The governance layer validates execution requests and returns typed decisions to the loop.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| ExecutionPermissionGate | application | Existing permission boundary | internal |
| Workspace action executor | infrastructure | Shell/edit/patch call sites | internal |
| Runtime configuration | infrastructure | Policy file/profile loading | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Policy location | domain model plus application evaluator | Keeps safety decisions testable. |
| Matching | prefix and executable rules first | Covers common shell posture without overfitting. |
| Output | typed decision with explanation | Supports trace and operator review. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| ExecutionPolicy | domain | Rule and decision vocabulary. |
| PolicyEvaluator | application | Deterministic rule evaluation. |
| PolicyConfigAdapter | infrastructure | Load local policy configuration. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Rule parse failure | config load | Reject policy and use safe default | Operator sees diagnostics. |
| Prompt unavailable | approval mode | Deny with explanation | Planner can choose alternative. |
| Command ambiguous | evaluator | Require prompt or deny | Evidence records ambiguity. |
