# Persist Replayable Sessions And Context - Software Design Description

> Introduce durable local-first thread, rollout, snapshot, compaction, and rollback state aligned with trace projections and recursive evidence.

**SRS:** [SRS.md](SRS.md)

## Overview

Create a local session persistence port that records recursive turn state independently from UI projections. The application layer writes normalized session records; infrastructure provides a file-backed or embedded database adapter.

## Context & Boundaries

Domain owns session record identity, event/evidence references, snapshot metadata, compaction lineage, and replay semantics. Application services decide when to persist. Infrastructure owns serialization, storage layout, and migrations.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Conversation history | infrastructure | Existing lightweight history baseline | internal |
| Trace recorder | application/infrastructure | Event/evidence source | internal |
| Workspace editor/executor | infrastructure | Snapshot triggers | internal |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Store posture | local-first durable store | Preserves privacy and replayability. |
| Replay | normalized event/evidence records | Avoids UI-specific transcript coupling. |
| Migration | version every record | Supports long-lived sessions. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| SessionStorePort | domain/application boundary | Persist and load session state. |
| SessionPersistenceService | application | Map runtime events into durable records. |
| LocalSessionStoreAdapter | infrastructure | File or embedded database storage. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Store unavailable | write failure | Continue session with warning evidence | Retry or export volatile trace. |
| Schema mismatch | version check | Refuse unsafe load | Run migration or start new session. |
| Snapshot missing | replay validation | Mark replay incomplete | Continue with available evidence. |
