# HTTP-Only Inference And Turn Runtime Migration - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver the HTTP-only model inference migration so action-selection and final-rendering model calls run through HTTP model clients, including local Ollama-hosted models. | board: VJZ034dF2 |
| MG-02 | Remove legacy Sift model-provider behavior by hard-failing old Sift inference config with an actionable `ollama:<model>` migration hint. | board: VJZ034dF2 |
| MG-03 | Collapse planner, synthesizer, and gatherer lane concepts across user-facing surfaces and internal Rust code so the codebase centers on turn runtime phases, model clients, retrieval, execution, evidence, and final rendering. | board: VJZ034dF2 |
| MG-04 | Leave Sift retrieval/indexing as a separate future decision; do not remove it as part of this inference migration. | board: VJZ034dF2 |

## Constraints

- Follow the approved migration recommendation from `.keel/epics/VJZ0tpZQJ/voyages/VJZ14yp0U/CLEANUP_MIGRATION_RECOMMENDATION.md`.
- Implement slices 1-5 from the recommendation in sealed, test-driven stories.
- Do not silently remap legacy `sift` model-provider settings to another provider.
- Use `ollama:<model>` as the canonical local HTTP model-provider form in docs, config examples, and migration hints.
- Keep local-first operation available through HTTP-hosted model services rather than paddles-owned model loading.
- Preserve Sift-backed retrieval/indexing until a separate mission explicitly decides its fate.
- Update owning docs in the same slice that changes runtime behavior or operator configuration.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied
- YIELD to human if removing Sift retrieval/indexing becomes necessary to complete an inference story.
- YIELD to human if a compatibility path would silently change model provider, model quality, latency, or deployment behavior.
