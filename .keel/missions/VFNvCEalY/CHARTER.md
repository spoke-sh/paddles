# Interpretation Context Quality And Visibility - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Users can see and validate what paddles understood from their guidance — categories, precedence, conflicts | board: VFNvFQPuA |
| MG-02 | Interpretation context is self-validating: the model checks for conflicts, missing context, and precedence gaps before committing | board: VFNvH5LxS |

## Constraints

- Interpretation quality improvements must not add new external dependencies
- The refinement loop must be bounded (max iterations) to prevent runaway cost
- Visibility changes must respect the existing verbosity tier system
- Guidance graph expansion depth/doc limits remain as architectural guardrails
- Changes to the interpretation pipeline must not regress existing test coverage

## Halting Rules

- HALT when both epics (VFNvFQPuA, VFNvH5LxS) are verified
- YIELD to human for UX review after TUI visibility changes land
- YIELD to human for prompt tuning after first recursive refinement integration
