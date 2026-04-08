# Make Entity Resolution Deterministic - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Deliver an active epic slice that gives Paddles a deterministic, self-discovering entity/path resolver for authored workspace files so edit-oriented turns converge on real files instead of hallucinating paths. | board: VGDNcabks |

## Constraints

- Keep the resolver local-first and self-discovering. Do not depend on IDE-fed context, editor state, or network services.
- Respect the authored workspace boundary everywhere the resolver participates, including `.gitignore`, generated directories, and execution-time edit rejection.
- Treat cached resolver state as machine-managed implementation detail. Cache invalidation must be explicit and safe so stale indices cannot steer edits into the wrong file.
- Prefer deterministic ranking and explicit ambiguity reporting over opaque heuristic guesses.

## Halting Rules

- DO NOT halt while epic `VGDNcabks` has any draft, planned, active, or verification-pending voyage/story work.
- HALT when epic `VGDNcabks` is complete and the active resolver path is verified through board-linked implementation evidence.
- YIELD to the human if the remaining design choice requires product direction on LSP adoption, cross-repo indexing, or non-local context sources.
