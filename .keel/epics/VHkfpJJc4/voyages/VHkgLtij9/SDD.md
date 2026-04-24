# Harden Workspace Editing And LSP Hands - Software Design Description

> Make workspace edits and semantic code intelligence safe, diagnosable, and planner-accessible through production-grade edit behavior and LSP-backed workspace actions.

**SRS:** [SRS.md](SRS.md)

## Overview

Extend the workspace hand as a hexagonal adapter around domain-level edit intent and semantic-inspection requests. Editing remains governed; LSP and formatter integrations are optional infrastructure capabilities.

## Context & Boundaries

Domain owns edit intent, semantic action types, diagnostics, and evidence contracts. Application services coordinate locks, formatter/diagnostic follow-up, and evidence attachment. Infrastructure owns filesystem, process, formatter, and LSP clients.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Local workspace editor | infrastructure | Existing edit execution | internal |
| Workspace action executor | infrastructure | Existing planner action bridge | internal |
| Optional LSP server | external/local process | Semantic code intelligence | configured |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| LSP posture | optional capability | Preserves basic local editing without language servers. |
| Edit safety | lock per file | Prevents conflicting writes in delegated or async paths. |
| Evidence | include diff plus diagnostics | Gives planner and operator runtime reality. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| WorkspaceEditService | application | Locking, preservation, formatter, diagnostics orchestration. |
| SemanticWorkspacePort | domain/application boundary | Typed LSP-like operations. |
| LspAdapter | infrastructure | Language server interaction. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Replacement ambiguous | matcher | Reject with candidate context | Planner can re-read and retry. |
| Formatter failure | exit status | Keep edit, attach diagnostic warning | Operator can inspect. |
| LSP unavailable | adapter status | Return unavailable semantic result | Planner falls back to search/read. |
