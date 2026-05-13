# Planner Lane Schema Adoption - Software Design Description

> Replace adapter-local action-schema blocks with the shared renderer and prove Sift and HTTP mocked turns receive the same canonical schema.

**SRS:** [SRS.md](SRS.md)

## Overview

Migrate provider prompt builders to consume the shared planner action schema
renderer. Sift/local and HTTP/remote lanes keep their transport-specific
instructions, but action names, JSON examples, required fields, semantic action
entries, external capability shape, and shared rules come from the same rendered
schema block.

## Context & Boundaries

The voyage changes prompt construction and tests only. Runtime execution,
provider authentication, and governance remain unchanged.

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
| Shared planner action schema renderer | application/domain helper | Canonical action schema text | prior voyage |
| Sift planner adapter | infrastructure adapter | Local planner prompt consumer | current crate |
| HTTP provider adapter | infrastructure adapter | Remote planner prompt consumer | current crate |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Transport separation | Keep transport mechanics in adapters | HTTP native tools and prompt envelopes differ from Sift |
| Schema comparison | Extract a named canonical schema block from mocked prompts | Proves lane parity while allowing surrounding prompt text to differ |
| Drift proof | Use mocked end-to-end turns where available | Acceptance requires runtime prompt path coverage |

## Architecture

Prompt builders call the shared renderer and embed the returned block. Tests
capture prompts from mocked Sift and HTTP turns, extract the canonical schema
block, and compare it exactly.

## Components

| Component | Purpose |
|-----------|---------|
| Sift prompt migration | Consume shared schema in local planner prompts |
| HTTP prompt migration | Consume shared schema in remote planner prompts |
| Schema block extractor | Test helper for comparing canonical blocks |
| Mocked-turn tests | Exercise real prompt paths for both lanes |

## Interfaces

Adapters receive rendered prompt text from the shared renderer. HTTP still
selects native-tool, structured JSON, or prompt-envelope transport instructions
outside the schema block.

## Data Flow

Planner request enters Sift or HTTP adapter. Adapter assembles context, asks the
shared renderer for the schema block, adds provider-specific transport text, and
sends the prompt to the model transport.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Sift prompt misses canonical block | mocked-turn test failure | fail CI | restore renderer call |
| HTTP prompt misses canonical block | mocked-turn test failure | fail CI | restore renderer call |
| Provider prompt needs transport-specific wording | review/test | keep outside canonical block | add adapter-local transport text only |
