# Bounded Context Self-Assessment - Software Design Description

> Implement bounded self-assessment for context compaction using planner evidence mechanisms

**SRS:** [SRS.md](SRS.md)

## Overview

Add compaction domain types (`CompactionRequest`, `CompactionPlan`, `CompactionDecision`) and an `assess_context_relevance()` function that evaluates a set of context artifacts for relevance using the same bounded evidence-gathering pattern as workspace planner tasks. Compacted summaries become first-class artifact envelopes that can be re-compacted.

## Context & Boundaries

```
┌──────────────────────────────────────────────────────────────┐
│            domain/model/compaction.rs (new)                   │
│  ┌──────────────────┐  ┌──────────────────┐                  │
│  │CompactionRequest │  │  CompactionPlan   │                  │
│  │ - target_scope   │  │  - decisions[]    │                  │
│  │ - relevance_query│  │    Keep/Compact/  │                  │
│  │ - budget         │  │    Discard        │                  │
│  └──────────────────┘  └──────────────────┘                  │
└──────────────────────────────────────────────────────────────┘
         ↑                        ↓
┌────────┴────────────┐  ┌───────┴──────────────────────┐
│ assess_context_     │  │ ArtifactEnvelope             │
│ relevance()         │  │ (wraps compacted summary     │
│ (bounded assessment)│  │  with ContextLocator)        │
└─────────────────────┘  └──────────────────────────────┘
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| ContextLocator | Internal | Locator for pre-compaction content | domain/model/context.rs (from VFOmKssE5) |
| ArtifactEnvelope | Internal | Wraps compacted summaries | paddles-conversation |
| PlannerBudget | Internal | Budget pattern reused for compaction | domain/ports/planning.rs |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Separate CompactionBudget from PlannerBudget | Dedicated type with max_steps only | Compaction needs fewer constraints than full planning |
| Assessment function, not trait | assess_context_relevance() as free function | Single implementation, no need for polymorphism yet |
| Compacted output is ArtifactEnvelope | Reuse existing envelope type | Enables recursive compaction — envelope can be compacted again |

## Components

### CompactionRequest (domain/model/compaction.rs)

```rust
pub struct CompactionRequest {
    pub target_scope: Vec<String>,      // artifact references to evaluate
    pub relevance_query: String,        // current context for relevance scoring
    pub budget: CompactionBudget,
}

pub struct CompactionBudget {
    pub max_steps: usize,               // bounded assessment steps
}
```

### CompactionPlan (domain/model/compaction.rs)

```rust
pub struct CompactionPlan {
    pub decisions: Vec<CompactionDecision>,
}

pub enum CompactionDecision {
    Keep { artifact_ref: String, priority: u8 },
    Compact { artifact_ref: String, summary: String },
    Discard { artifact_ref: String, reason: String },
}
```

### assess_context_relevance() (domain/services/compaction.rs)

Takes a `CompactionRequest`, evaluates each artifact's relevance to the `relevance_query` within the budget, and returns a ranked `CompactionPlan`.

## Data Flow

1. Caller provides `CompactionRequest` with artifacts to evaluate and current context
2. `assess_context_relevance()` iterates artifacts within budget.max_steps
3. For each artifact: score relevance to query, decide keep/compact/discard
4. Compacted artifacts get summaries wrapped in `ArtifactEnvelope` with `ContextLocator` to original
5. Return `CompactionPlan` with all decisions

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Budget exhausted before all artifacts assessed | Step counter reaches max_steps | Return partial plan with remaining artifacts marked as Keep | Caller can re-assess with larger budget |
| Artifact reference not found | Lookup fails | Skip artifact with Discard { reason: "not found" } | Continue assessment |
