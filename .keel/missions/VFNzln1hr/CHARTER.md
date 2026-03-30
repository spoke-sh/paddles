# Recursive Context Architecture - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Document and formalize the context topology: how subcontext components (evidence budgets, artifact envelopes, thread summaries, operator memory, planner loop state) discover, address, and communicate with each other today, and where the seams and blind spots are. | manual: foundational doc reviewed and accepted |
| MG-02 | Establish transit-native context addressing so that any context component can navigate to related context through transit lineage (streams, branches, merge records) rather than relying solely on in-memory structural wiring and factory-time assembly. | board: pending |
| MG-03 | Design recursive self-assessing compaction where the system evaluates its own context state and decides what to compact, promote, archive, or surface, using the same bounded planner/evidence mechanisms it uses for workspace tasks. | board: pending |
| MG-04 | Model context pressure, staleness, and relevance decay as first-class signals within the constraints and capabilities framework, so the system can report and respond to context budget exhaustion the same way it handles planner budget exhaustion. | board: pending |
| MG-05 | Formalize the unbounded context tier model: inline (truncated artifacts) to transit streams to sift indexes to filesystem to beyond, with traversal and resolution semantics so the system can reach any depth on demand without holding everything in working memory. | board: pending |

## Constraints

- Transit is the backbone. Context addressing and traversal must flow through transit lineage primitives (streams, branches, checkpoints, merge records) rather than inventing a parallel graph.
- Do not centralize context into a single monolithic structure. The architecture must remain distributed across components that can operate and compact independently.
- Compaction must be recursive and composable. A compacted summary is itself a context artifact that can be further compacted, not a terminal state.
- Preserve the domain boundary. Transit-core types must not leak through paddles domain ports. Context addressing uses paddles-owned types that align with transit semantics.
- Self-assessment must be bounded. The system evaluating its own context uses the same budget and capability constraints as any other planner task, preventing infinite meta-recursion.
- Context resolution must be lazy and pull-based. Components retrieve related context on demand through addressable locators, not by eagerly loading everything into memory.
- Local-first and fail-closed. Context traversal degrades honestly when a tier is unavailable rather than blocking or panicking.
- Do not break existing turn flow. The current evidence-budget and artifact-envelope mechanisms continue to work; this mission extends and connects them, not replaces them.

## Halting Rules

- DO NOT halt while any MG-* goal with `board:` verification has unfinished board work.
- HALT when all goals are satisfied and paddles has a documented, transit-backed context architecture where components can discover each other, the system can self-assess and compact its own context recursively, and context pressure is modeled within the capabilities framework.
- YIELD to human when MG-01 foundational documentation is ready for review, before proceeding to implementation goals.
- YIELD if recursive compaction cannot be bounded safely within the existing planner budget contract without introducing unbounded meta-recursion.
