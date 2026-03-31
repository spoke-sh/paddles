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

## Context Tier Model

The system accesses context through four tiers, each with distinct boundaries, storage characteristics, and traversal semantics.

| Tier | Boundary | Storage | Access Pattern | Resolution |
|------|----------|---------|----------------|------------|
| **Inline** | Character-limited content embedded in working memory | `ArtifactEnvelope.inline_content` | Direct read, zero-cost | Identity (content is the value) |
| **Transit** | Full trace records persisted in streams | `TransitTraceRecorder` streams keyed by `TaskTraceId` + `TraceRecordId` | Replay task stream, find record by ID | `TransitContextResolver::resolve(Transit { task_id, record_id })` |
| **Sift** | Indexed evidence in retrieval indexes | Sift indexes (out of current scope) | Query-based retrieval | Not yet implemented; returns explicit error |
| **Filesystem** | Workspace files on disk | Local filesystem at workspace root | Path-based read | `tokio::fs::read_to_string(path)` |

### Tier Boundaries

- **Inline → Transit**: Content exceeding the inline character limit is truncated with a `[truncated]` suffix. The `ArtifactEnvelope` carries a `ContextLocator::Transit` pointing to the full record in the transit stream.
- **Transit → Filesystem**: Transit records may reference workspace file paths via `ContextLocator::Filesystem`. Resolution reads the file directly.
- **Sift tier**: Reserved for future indexed retrieval. Resolution attempts return an explicit unsupported error.

### Traversal Rules

1. **Typed addressing**: Every `ContextLocator` variant encodes its target tier via `locator.tier() -> ContextTier`. Consumers route resolution through the `ContextResolver` trait.
2. **Lazy pull-based resolution**: No tier eagerly loads content from a deeper tier. Resolution occurs only when a consumer calls `resolver.resolve(&locator)`.
3. **Local-first ordering**: Inline content is returned directly. Transit replays local streams. Filesystem reads local disk. Remote tiers (sift) are attempted last.
4. **Fail-closed degradation**: When a tier is unavailable or a record is missing, resolution returns an explicit `anyhow::Error` naming the tier and locator details. Consumers degrade to truncated inline content.

## Target Architecture Goals

- **Transit-Native Addressing**: Move from structural wiring to addressable locators backed by transit lineage.
- **Recursive Self-Assessing Compaction**: Use the planner itself to evaluate and compact its own context state.
- **Unbounded Tier Model**: Formalize the transition from inline context to transit streams, sift indexes, and the filesystem.
- **Context Signals**: Integrate context pressure and relevance decay into the capability framework.
