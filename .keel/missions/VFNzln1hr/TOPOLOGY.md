# Context Topology - Paddles

This document formalizes the current subcontext components in `paddles`, their discovery mechanisms, and the identified seams and blind spots in the context architecture.

## Subcontext Components

| Component | Responsibility | implementation | Discovery / Access | Constraints |
|-----------|----------------|----------------|-------------------|-------------|
| **Evidence Budgets** | Controls the volume of evidence gathered for synthesis | `EvidenceBudget` in `src/domain/ports/context_gathering.rs` | Passed through `ContextGatherRequest` | Default: 8 items, 1200 summary chars, 600 snippet chars |
| **Artifact Envelopes** | Inline truncation for large trace records | `paddles-artifact://` locators in `crates/paddles-conversation` | Injected into trace records | Logical IDs used to retrieve full content from recorder |
| **Thread Summaries** | Contextual link between parent and child threads | 80-char trimmed pairs on merge | Passed through `PlannerRequest::recent_thread_summary` | Strict 80-char prefix + ellipsis |
| **Operator Memory** | High-level guidance and project conventions | `AgentMemory` in `src/infrastructure/adapters/agent_memory.rs` | Loaded from `AGENTS.md` hierarchy | 12,000 character cap per file |
| **Planner Loop State** | Working memory for the recursive investigation | `PlannerLoopState` in `src/domain/ports/planning.rs` | Accumulated in `RecursiveExecutionLoop` | Bounded by `PlannerBudget` and `step_limit` |

## Discovery and Communication

Today, context components are largely isolated silos, connected only by manual wiring at factory-assembly time:

- **Factory Assembly**: `MechSuitService` and `PlannerLoopContext` act as the primary orchestrators, manually gathering interpretation context, recent turns, and loop state into flat structures.
- **`build_planner_prior_context()`**: This function is the primary context assembly point, flattening diverse signals (interpretation, turns, steps, pending branches) into a `Vec<String>` for the planner.
- **Lack of Runtime Discovery**: There is no mechanism for one context component to "find" or address another at runtime. For example, the planner cannot navigate from a thread summary back to the full context of the parent thread without explicit orchestrator support.

## Seams and Blind Spots

### 1. Transit as a Write-Only Sink
Transit-core is currently used as a durable record of what *happened*, but it is not utilized during a turn to retrieve what *is*. The rich lineage stored in transit (streams, branches, merge records) is unavailable to the active planner for context traversal.

### 2. Context Addressing vs. Structural Wiring
Components are "pushed" into context by the orchestrator. There is no "pull-based" addressing. If the planner sees an `artifact-123` ID, it cannot resolve it to a location in transit or on the filesystem without going through the central orchestrator's narrow ports.

### 3. Mechanical vs. Semantic Compaction
Compaction is currently performed using fixed character limits (e.g., 12k for `AGENTS.md`, 80 chars for summaries). There is no self-assessment of relevance or importance. A "compaction" is a destructive operation that loses signal, rather than a recursive summary that preserves addressable depth.

### 4. Context Pressure Blindness
The system tracks `PlannerBudget` (steps, tool calls), but it has no first-class concept of "Context Budget". It cannot signal that it is under "context pressure" or decide to promote/archive components based on available token/char limits.

## Target Architecture Goals

- **Transit-Native Addressing**: Move from structural wiring to addressable locators backed by transit lineage.
- **Recursive Self-Assessing Compaction**: Use the planner itself to evaluate and compact its own context state.
- **Unbounded Tier Model**: Formalize the transition from inline context to transit streams, sift indexes, and the filesystem.
- **Context Signals**: Integrate context pressure and relevance decay into the capability framework.
