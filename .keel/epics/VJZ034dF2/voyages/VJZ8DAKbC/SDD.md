# Migrate Provider Preferences To Turn Runtime Config - Software Design Description

> Replace lane-shaped provider preferences with turn-runtime model-client preferences, use Ollama as the canonical local HTTP example, and keep legacy config readable only for migration.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage removes lane-shaped provider preferences from the active
configuration model. The runtime should expose model-client choices as turn
runtime preferences: action selection, final rendering, retrieval, and any
other explicit turn phase. Legacy lane files remain readable only so existing
users get deterministic migration behavior.

## Context & Boundaries

The previous runtime lane config encoded planner, synthesizer, and gatherer as
parallel lanes. The target preference model should teach how the turn runtime is
actually assembled. This voyage does not delete retrieval providers or choose a
specific Ollama model name; it standardizes the provider reference form as
`ollama:<model>`.

```
┌─────────────────────────────────────────┐
│         Turn Runtime Preferences        │
│                                         │
│  action_selection -> model client       │
│  final_rendering  -> model client       │
│  retrieval        -> retrieval provider │
└─────────────────────────────────────────┘
        ↑               ↑
 legacy lanes      new writes
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Runtime config parser | Rust config | Read legacy and new preference forms | repo-local |
| Preference persistence | Rust infrastructure | Write new turn-runtime preference shape | repo-local |
| Credential store | Rust infrastructure | Preserve HTTP auth boundaries | repo-local |
| `CONFIGURATION.md` | project doc | Own file names, precedence, and examples | repo-local |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| New config language | Turn runtime preferences | Matches the target runtime architecture. |
| Legacy lane config | Read-only migration input | Avoids breaking existing files while stopping new lane-shaped writes. |
| Ollama examples | `ollama:<model>` | Keeps local HTTP examples concrete without picking a model for every user. |
| Provider credentials | Preserve existing boundaries | This migration should not weaken HTTP auth requirements. |

## Architecture

The parser should normalize legacy lane-shaped fields into the new turn-runtime
preference model, recording warnings or migration hints when appropriate. Any UI
or command that persists preferences should emit only the new shape. Runtime
construction consumes the normalized turn-runtime preferences.

## Components

| Component | Purpose | Behavior |
|-----------|---------|----------|
| Turn runtime preference schema | Canonical provider preference state | Names action selection, final rendering, and retrieval explicitly |
| Legacy lane parser | Compatibility input | Reads old fields and reports migration guidance |
| Preference writer | Persistence boundary | Writes only new turn-runtime config |
| Docs | Operator contract | Explains precedence and `ollama:<model>` examples |

## Interfaces

The new preference shape should avoid `planner`, `synthesizer`, `gatherer`, and
`lane` as canonical field names. Compatibility aliases may exist only at read
time and should point to their replacement phase names.

## Data Flow

1. Read authored config and persisted preferences.
2. Normalize legacy lane-shaped fields into turn-runtime preferences.
3. Reject legacy Sift model-provider values with the ADR migration hint.
4. Build model clients and retrieval providers from normalized preferences.
5. Persist future changes only in the turn-runtime preference shape.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Legacy lane config contains `sift` model inference | Normalization | Return hard-fail migration hint | Configure `ollama:<model>` or another HTTP provider |
| New config write attempts lane-shaped fields | Persistence tests | Fail test/build until writer is updated | Write turn-runtime fields only |
| Required HTTP provider credentials are missing | Credential validation | Fail closed with provider-specific message | Configure credentials or choose a provider without auth |
