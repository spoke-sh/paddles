# Replace Autonomous Sift Planning With Direct Retrieval - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Replace the `sift-autonomous` gatherer path with a direct sift-backed retrieval boundary so paddles remains the only recursive planner during search-heavy coding turns. | board: VFV0VmEj0 |
| MG-02 | Make retrieval execution observable enough that long-running sift work explains what stage is active and why it is taking time. | manual: direct retrieval progress surfaces concrete execution stages and avoids opaque autonomous planner states |
| MG-03 | Leave the mission ready for immediate execution without unresolved board drift. | manual: mission activates cleanly and its first voyage is planned with backlog-ready stories |

## Constraints

- Preserve paddles-owned recursive planning; do not reintroduce nested autonomous planning through another adapter.
- Use sift as a library/backend for retrieval execution, indexing, and ranking rather than as an end-user planner.
- Keep user-facing progress updates grounded in concrete execution stages instead of opaque planner internals.
- Defer large upstream sift redesigns unless they are required to support the direct retrieval boundary.

## Halting Rules

- DO NOT halt while any MG-* goal has unfinished board work
- HALT when all MG-* goals with `board:` verification are satisfied and the active mission can be executed through planned stories
- YIELD to human when only `metric:` or `manual:` goals remain
