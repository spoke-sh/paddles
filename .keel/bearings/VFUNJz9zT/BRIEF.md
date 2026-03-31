# Adaptive Interpretation Context Refinement — Brief

## Hypothesis

Interpretation context assembled once at turn start becomes stale as the planner accumulates evidence. A system that periodically re-evaluates what guidance, constraints, and capabilities are relevant — based on conversation state, evidence gathered, and thread lineage — would produce better-grounded planner decisions and fewer wasted steps.

## Problem Space

Today, `derive_interpretation_context` runs once per turn. It loads AGENTS.md, derives a guidance subgraph, extracts tool hints and decision procedures, and freezes. The planner then works within that static context for all its steps.

This creates three problems:

1. **Stale relevance.** The guidance derived from the user prompt at turn start may not match what the planner discovers mid-investigation. A search for CI configuration doesn't need the same guidance as a code edit.

2. **No constraint negotiation.** The system never asks "given what I've learned, should my constraints or capabilities change?" The planner budget, tool access, and retrieval strategy are fixed regardless of what evidence reveals.

3. **No cross-turn learning.** In an interactive session, each turn re-derives context from scratch. Thread history, prior evidence, and conversation patterns don't inform what guidance is promoted or demoted.

## Proposed Architecture

### Refinement Triggers

Instead of one-shot assembly, interpretation context would be re-evaluated at defined trigger points:

| Trigger | When it fires | What it re-evaluates |
|---------|--------------|---------------------|
| **Evidence threshold** | After N evidence items accumulate | Guidance relevance, tool hint priority |
| **Thread transition** | When a thread decision creates/merges a branch | Full context re-derivation from thread state |
| **Constraint pressure** | When context pressure reaches High or Critical | Budget allocation, retrieval strategy |
| **Capability discovery** | When a tool call reveals new workspace structure | Tool hints, available actions |
| **Periodic heartbeat** | Every K planner steps | Lightweight relevance check |

### Negotiation Model

At each trigger, the system would:

1. Present the model with the current interpretation context + accumulated evidence
2. Ask: "Given what you now know, which guidance is still relevant? Should any constraints change?"
3. The model responds with a structured refinement: promote/demote guidance, adjust budget allocation, add/remove tool hints
4. The planner loop continues with the updated context

This is not a full re-derivation — it's a delta negotiation that keeps the cheap parts and re-evaluates the expensive parts.

### Transit Thread Integration

Thread lineage in transit provides a natural trigger source:

- When a new child thread is created, the interpretation context should reflect the narrowed scope
- When threads merge back, guidance from both branches should reconcile
- Thread summary content (80-char trimmed pairs) could inform which guidance documents are still relevant
- The conversation replay view already tracks thread structure — it needs to feed back into interpretation

### Background Refinement

For long-running turns, refinement could happen concurrently:

1. The planner continues executing its current step
2. A background task re-evaluates interpretation context using the latest evidence
3. The next planner step uses the refined context
4. This prevents refinement from adding latency to the critical path

## Success Criteria

- [ ] Planner makes fewer redundant investigation steps when context is refined mid-turn
- [ ] Thread transitions produce context that reflects the narrowed or broadened scope
- [ ] Context pressure signals trigger meaningful constraint adjustments (not just informational)
- [ ] Refinement overhead stays under 2 seconds per trigger (amortized across the turn)
- [ ] No regression in turn latency for simple direct-response turns

## Open Questions

- What is the right granularity for the negotiation response schema? Full re-derivation vs. delta patches?
- Should refinement be synchronous (blocking the next planner step) or asynchronous (background)?
- How do we prevent refinement oscillation — context changing back and forth between steps?
- Should the first interpretation still be model-derived, or should it start with a cheaper heuristic and refine up?
- What transit events should serve as thread transition triggers? Only explicit decisions, or also implicit checkpoint boundaries?
- How does this interact with compaction? Should refinement and compaction share the same trigger points?
