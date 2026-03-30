# Recursive Context Architecture - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-03-30

### Sealing move: Mission Decomposition and Context Topology Analysis

- **Context Audit**: Mapped the full context topology across paddles. Identified five subcontext components that currently manage context independently: evidence budgets (static caps in `ContextGatherRequest`), artifact envelopes (inline truncation with `paddles-artifact://` locators), thread summaries (80-char trimmed pairs on merge), operator memory (cascading `AGENTS.md` with 12k char cap), and planner loop state (`PlannerLoopState` accumulating steps/evidence/branches).

- **Communication Gap Analysis**: These components communicate through structural wiring at factory-assembly time via `MechSuitService` and `PlannerLoopContext`. The `build_planner_prior_context()` function is the closest thing to a discovery mechanism — it manually assembles interpretation, recent turns, planner steps, and pending branches into a flat `Vec<String>`. Components cannot find each other at runtime; they only see what the orchestrator explicitly passes them.

- **Transit Opportunity**: Transit already provides the primitives needed — `LocalEngine` supports streams, branches with lineage metadata, checkpoints, and replay. The `TransitTraceRecorder` adapter already maps conversation threads to transit branch streams. But today transit is write-only from paddles' perspective: records go in, nothing reads them back during a turn except full replay for `ConversationReplayView`. The merge semantics (backlink, summary, merge modes) exist in paddles' thread model but aren't yet leveraging transit's native branch resolution.

- **Unbounded Context Insight**: Through the combination of paddles (in-memory working context), transit (durable lineage streams), sift (autonomous retrieval with graph-mode exploration), and the filesystem (workspace artifacts), the system already has access to effectively unlimited context. The missing piece is not storage but traversal — connecting these tiers with addressable locators and lazy resolution so the system can navigate from a truncated inline artifact to its full content in transit, to related evidence in sift, to source files on disk.

- **Compaction Design Direction**: Current "compaction" is purely mechanical — fixed char limits, no awareness of relevance or staleness. Recursive self-assessing compaction would treat context evaluation as a planner task: the system uses a bounded evidence-gathering pass over its own context to decide what to compact, promote to a higher tier, or discard. A compacted summary becomes a new artifact envelope that can itself be compacted later — compaction is not a terminal operation but a recursive one.

- **Mission Goals Defined**: Decomposed into five goals: (MG-01) document the context topology and its seams, (MG-02) transit-native addressing so components can find each other, (MG-03) recursive self-assessing compaction, (MG-04) context pressure as a capability signal, (MG-05) formalize the unbounded tier model with traversal semantics.
