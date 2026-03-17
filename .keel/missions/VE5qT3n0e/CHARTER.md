# The Architectural Lattice - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Establish DDD/Hexagonal directory structure | board: VE5qX07aD |
| MG-02 | Migrate Boot and Inference logic to Domain/Application layers | board: VE5qX07aD |
| MG-03 | Verify full system integrity after architectural shift | board: VE5qX07aD |

## Constraints

- Must maintain existing functionality (boot calibration, dogma, interactive loop).
- Must adhere to the new `ARCHITECTURE.md` specialized for Paddles.
- No new features during this refactor; focus purely on structural integrity.

## Halting Rules

- HALT when `keel doctor` reports nominal and `just paddles` executes successfully.
