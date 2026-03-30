# Refinement Loop Core - Software Design Description

> Add typed guidance categories, precedence extraction, conflict detection, and coverage gap validation

**SRS:** [SRS.md](SRS.md)

## Overview

Extend the interpretation context schema and the model prompt to produce structured understanding: what kinds of guidance exist (categories), which documents take precedence (precedence chain), where they conflict (conflicts), and what's missing (gaps). The first three are extensions to the existing single-pass assembly. The gap validation is a separate model call.

## Components

### GuidanceCategory enum (planning.rs)

`Rules | Conventions | Constraints | Procedures | Preferences`. Deserialized from model response with unknown variants silently dropped.

### Extended InterpretationContext fields

- `categories: Vec<GuidanceCategorySummary>` — `{ category: GuidanceCategory, count: usize, sources: Vec<String> }`
- `precedence_chain: Vec<PrecedenceEntry>` — `{ source: String, rank: usize, scope_label: String }`
- `conflicts: Vec<GuidanceConflict>` — `{ sources: Vec<String>, description: String, resolution: String }`

All default to empty Vec when the model doesn't populate them.

### Extended interpretation prompt (sift_agent.rs)

Add three sections to `build_interpretation_context_prompt`:
1. "Classify each guidance item by category..."
2. "State the precedence chain given the document loading order..."
3. "Identify any conflicts between guidance documents..."

The JSON schema grows to include `categories`, `precedence_chain`, and `conflicts` arrays.

### Validation function (sift_agent.rs)

`validate_interpretation_coverage(context: &InterpretationContext, prompt: &str, model: &dyn ...) -> Vec<CoverageGap>`

Sends one model call: "Given this interpretation context and user request, what areas have no guidance coverage?" Returns `Vec<{ area: String, suggestion: String }>`.

## Data Flow

1. `build_interpretation_context_prompt` includes category/precedence/conflict instructions
2. Model returns extended JSON envelope
3. `interpretation_context_from_envelope` parses new fields alongside existing ones
4. Validation function called separately after assembly (not wired into main loop yet)

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Categories in same model call | Yes | No additional cost — already assembling context |
| Validation as separate call | Yes | Clean separation; can be skipped when not needed |
| Empty defaults | All new fields default empty | Graceful degradation |
