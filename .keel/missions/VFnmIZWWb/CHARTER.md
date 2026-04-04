# Make Workspace Editor Diffs Visible - Charter

Archetype: Strategic

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Make applied workspace edits visible as first-class diffs across the runtime model, web UI, and TUI so operators can tell when Paddles actually used the workspace editor. | board: VFnmIbFW2 |

## Constraints

- Preserve the provider-agnostic `WorkspaceEditor` boundary. Diff visibility must come from workspace editor results and runtime events, not provider-specific rendering paths.
- Land the capability in sealed slices: edit artifact emission, diff synthesis, then cross-surface rendering and contracts.
- Keep the web and TUI representations semantically aligned around the same applied-edit artifact so one surface does not drift from the other.

## Halting Rules

- DO NOT halt while epic `VFnmIbFW2` still has undispatched voyage or story work needed to make workspace edits visually explicit.
- HALT when epic `VFnmIbFW2` is done and the applied-edit diff is visible in both the web runtime stream and the TUI transcript stream with proof attached.
- YIELD to human if the required diff presentation conflicts with desired UX semantics or if the user wants a different visual diff idiom than the current Codex/Claude/Gemini-style direction.
