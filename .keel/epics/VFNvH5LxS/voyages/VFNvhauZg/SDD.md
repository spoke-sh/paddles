# Refinement Loop Integration - Software Design Description

> Wire the refinement loop into the application layer, emit progress events, and add coverage confidence

**SRS:** [SRS.md](SRS.md)

## Overview

After `derive_interpretation_context`, the application layer runs a bounded refinement cycle: validate coverage gaps → if gaps found, re-expand guidance graph targeting gaps → re-assemble context. Capped at 2 additional model calls. Emits TurnEvents at each stage. Falls back to single-pass result on any failure. Adds a CoverageConfidence field to InterpretationContext.

## Components

### Bounded re-expansion (sift_agent.rs)

When `validate_interpretation_coverage` returns gaps, pass gap suggestions as hints to `expand_interpretation_guidance_graph`. The hint string is appended to the graph expansion prompt so the model targets missing areas. Bounded to 1 cycle — even if re-expanded result still has gaps, no further expansion.

### Application layer wiring (application/mod.rs)

After `derive_interpretation_context` (~line 1020):
1. Call `validate_interpretation_coverage(context, prompt)` — 1 model call
2. If gaps empty → set confidence = High, skip re-expansion
3. If gaps found → emit InterpretationValidated event, call re-expand + re-assemble — 1 model call
4. If re-assembly succeeds → set confidence = Medium, emit InterpretationRefined
5. If re-expansion fails → keep original context, set confidence = Low
6. On any error in steps 1-5 → keep original context, set confidence = Normal (no signal)

### CoverageConfidence (planning.rs)

`High | Medium | Low` enum. Default = High (optimistic when validation isn't run, e.g. fallback path).

### TurnEvent variants (turns.rs)

- `InterpretationValidated { gap_count: usize, confidence: String }` — min_verbosity = 1
- `InterpretationRefined { gaps_before: usize, gaps_after: usize, confidence: String }` — min_verbosity = 0

## Data Flow

```
derive_interpretation_context
  ↓
validate_interpretation_coverage (1 model call)
  ↓ gaps?
  ├─ no gaps → confidence=High, done
  └─ gaps → re-expand graph with hints → re-assemble (1 model call)
      ├─ success → confidence=Medium
      └─ failure → keep original, confidence=Low
```

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Validation model call fails | Result::Err | Skip refinement | Original context, no confidence signal |
| Re-expansion model call fails | Result::Err | Keep original context | confidence = Low |
| Re-assembly parse fails | None from parse | Keep original context | confidence = Low |
