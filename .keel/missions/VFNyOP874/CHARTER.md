# Interactive Sift Search Progress - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Users see real-time progress during sift indexing and graph search instead of a silent wait | board: VFNyZ12IX |
| MG-02 | Upstream sift exposes a progress callback contract that paddles can consume | manual: sift crate exposes progress callback in search_autonomous API |

## Constraints

- Progress reporting must not add significant overhead to the search path
- The paddles-side implementation must degrade gracefully if sift doesn't yet support callbacks (show elapsed time, not fake progress)
- The synchronous search_autonomous call must move off the tokio runtime thread (spawn_blocking)
- Upstream sift changes are in a separate repo and require coordination

## Halting Rules

- HALT when MG-01 epic is verified and upstream sift requirements are documented
- YIELD to human for upstream sift API design decisions
- YIELD to human for sift repo access/coordination
