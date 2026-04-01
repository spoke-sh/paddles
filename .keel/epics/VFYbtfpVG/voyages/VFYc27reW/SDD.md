# Shared Conversation Transcript Plane - Software Design Description

> Replace interface-local transcript assembly with a shared conversation-scoped transcript projection and use it as the transcript source for TUI and web surfaces.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage keeps the existing turn execution path intact and adds an explicit transcript read model beside it. The application layer becomes responsible for projecting a single conversation transcript from durable trace-backed records and for notifying listeners when that conversation transcript changes. TUI and web then become projection clients: they bootstrap from conversation-scoped replay, subscribe to transcript-plane updates, and use progress events only for live execution detail.

## Context & Boundaries

### In Scope

- conversation identity/attachment for cross-surface prompt entry
- transcript replay/update APIs in the application layer
- web and TUI migration onto the transcript plane

### Out of Scope

- changes to planner behavior
- trace visualization redesign
- remote multi-user sync

```
┌────────────────────────────────────────────────────────────────────┐
│                           This Voyage                             │
│                                                                    │
│  prompt source ──> command path ──> durable trace records          │
│       │                │                    │                       │
│       │                │                    └─> transcript replay   │
│       │                │                    └─> transcript update   │
│       │                │                                             │
│       └────────────────────────────────────> TUI / web projection    │
└────────────────────────────────────────────────────────────────────┘
          ↑                                         ↑
     progress plane                            transcript plane
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `MechSuitService` application layer | internal | Canonical command path plus new transcript replay/update interfaces | current |
| Trace recorder / replay model | internal | Durable prompt/completion substrate for transcript projection | current |
| TUI adapter | internal | Terminal transcript and progress rendering client | current |
| Web adapter | internal | Browser transcript and progress rendering client | current |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Canonical transcript substrate | Start with a trace-backed conversation transcript projection | The trace model already stores prompts and completions, making this the lowest-friction unification seam |
| Progress vs transcript | Keep them as separate planes | Progress timing is race-prone and should not control transcript visibility |
| Conversation key | Reuse stable conversation/task identity across surfaces | Prevents each UI from inventing its own transcript namespace |
| Surface behavior | Allow brief optimistic UI rendering only if it reconciles to the canonical transcript plane | Preserves responsiveness without reintroducing divergent transcript truth |

## Architecture

1. Prompt entry remains routed through `process_prompt_in_session_with_sink(...)`.
2. Trace-backed prompt and completion records are projected into a conversation-scoped transcript replay model.
3. Transcript updates are emitted through a dedicated observer/signal path distinct from `TurnEvent`.
4. Web and TUI subscribe to the same transcript plane for bootstrap and live convergence.
5. Existing progress events continue to power trace/activity rendering only.

## Components

`TranscriptProjection`
: Application-layer read model that replays prompt/completion rows for one conversation and can recover state after missed updates.

`ConversationAttachment`
: Shared rules for choosing or reusing the conversation/task identity across prompt sources.

`TranscriptUpdateSignal`
: Dedicated invalidation or delta path that tells surfaces when to refresh or append transcript state.

`WebTranscriptClient`
: Browser-side projection client that replaces replay polling and DOM-local transcript truth.

`TuiTranscriptClient`
: TUI-side projection client that replaces local-only transcript append and external trace scraping.

## Interfaces

Candidate internal interfaces:

- `replay_conversation_transcript(task_id) -> TranscriptReplay`
- `register_transcript_observer(...)` or `subscribe_transcript_updates(task_id)`
- `attach_to_conversation(task_id)` for surfaces that join an existing conversation

Candidate adapter contracts:

- Web transcript bootstrap endpoint scoped to one conversation/session
- Web live update event or invalidation notification scoped to one conversation
- TUI observer callback that triggers conversation-scoped transcript refresh or append

## Data Flow

1. A surface submits a prompt against a conversation identity.
2. The application command path executes the turn and records durable trace-backed prompt/completion data.
3. The application emits transcript-plane update notification(s) for that conversation.
4. TUI and web receive the update and reconcile their displayed transcript from the canonical conversation projection.
5. In parallel, `TurnEvent` progress continues to update activity/trace views.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Surface misses a live transcript update | Transcript version or replay signature does not match local state | Trigger conversation-scoped replay | Rebuild transcript from canonical replay without page reload or TUI restart |
| Conversation identity mismatch between surfaces | Submitted turn appears under a different task than expected | Surface shows explicit mismatch/fallback behavior and avoids merging transcripts silently | Re-attach to the intended conversation identity |
| Trace-backed projection lacks needed transcript fields | Replay tests fail or projection cannot produce canonical rows | Stop migration and extend the substrate before removing old paths | Add missing durable transcript records or dedicated journal support |
