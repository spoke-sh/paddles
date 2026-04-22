# Stabilize Stream Rendering And Recursive Loop Architecture - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that makes stream rendering, recursive execution, and projection ownership coherent by preserving typed render documents end-to-end, enforcing one application-owned recursive control plane, and restoring domain/application/infrastructure boundaries around read models and presentation. | board: VHURpL4nG |

## Constraints

- Preserve local-first runtime constraints. Do not introduce new hosted orchestration, remote state services, or network dependencies to fix stream/rendering coherence.
- Keep `RenderDocument` and typed authored responses as the canonical answer contract across live streaming, durable trace storage, and replayed transcript projection.
- Collapse recursive control into one application-owned loop. Adapters may plan, gather, or author, but they must not own competing tool-execution or replanning loops.
- Keep projection and presentation concerns out of the domain core. Domain types may describe events and invariants, but UI-facing read models and formatting belong in application or infrastructure layers.
- Maintain operator visibility. Stream, transcript, forensic, and manifold surfaces must remain derivable from the same durable trace/projection spine.
- Update the owning docs and board artifacts in the same slices that change behavior or architectural boundaries.

## Halting Rules

- DO NOT halt while epic `VHURpL4nG` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VHURpL4nG` is complete and the runtime is backed by board-linked evidence for canonical render persistence, a single recursive control plane, and hexagonal read-model/projection boundaries.
- YIELD to the human if the remaining decision requires product direction on operator-facing streaming behavior, cross-surface UX semantics, or whether any transitional compatibility layer should survive the refactor.
