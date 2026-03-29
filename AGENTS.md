# AGENTS.md

Shared guidance for AI agents operating the `paddles` mech suit.

> Foundational stack position: `1/8`
> Read this document first. Then move to [INSTRUCTIONS.md](INSTRUCTIONS.md).

## Operational Guidance

You are an operator within the `paddles` harness. Keel is an engine with strict constraints (see [POLICY.md](POLICY.md)). Your primary responsibility is to execute tactical moves that advance the board state while maintaining mech-suit integrity and local-first runtime constraints.

### Core Principles
1. **Maintain Calibration**: The boot sequence (credits, weights, biases) is foundational. Ensure changes to `src/main.rs` never weaken Constitution or Dogma validation.
2. **Local First**: Prioritize local inference capacity via `candle`. Avoid introducing network dependencies into the core execution loop.
3. **Interpret Before Routing**: Operator memory should influence first-pass interpretation before the harness commits to a route. Do not commit to a path before the relevant `AGENTS.md` and foundational context are assembled.
4. **Model-Directed Next Action**: After interpretation context is assembled, the model should choose the next bounded action from a constrained schema. The controller should validate and execute that action safely instead of heuristically guessing the route.
5. **Recursive Planning Before Final Answer**: Difficult workspace questions should improve through bounded recursive resource use. Planner work and final synthesis are different roles and should not be collapsed into one brittle one-shot path.
6. **Visible Turns**: The interactive REPL should render a default Codex-style action stream so interpretation, recursive actions, routing, retrieval, tools, fallbacks, and synthesis are observable without extra flags.
7. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input.
8. **Grounded Answers**: Repository answers should cite source files by default and admit insufficient evidence instead of improvising unsupported claims.
9. **Pacemaker Hygiene**: Monitor system stability with `keel health --scene`, `keel flow --scene`, `keel doctor`, and dirty-worktree state. The pacemaker is derived from repository activity; uncommitted energy in the worktree is tactical debt that should be closed autonomously by landing the sealing commit.
10. **Notification Discipline**: Ping the human operator ONLY when you need input on design direction or how the application behaves. Resolve technical drift and tactical moves autonomously.

### Runtime Routing Contract

- The model should choose the next bounded action from a controller-defined schema; the controller validates and executes it safely.
- The primary mech-suit path now assembles interpretation context and asks the **planner lane** to choose the first bounded action before route selection.
- The **synthesizer lane** remains the final answer path for direct responses and grounded responses after planner work.
- Turns should improve through recursive context work instead of project-specific hardcoded intents or top-level heuristic routing.
- Repository-question answers should include file citations by default.
- TTY interactive sessions should expose a default transcript TUI with visible turn events rather than hiding runtime behavior behind verbose-only diagnostics.
- Interactive sessions should keep a paddles-owned conversation root so steering prompts can be classified into mainline continuation, child-thread splits, or merge-back outcomes with durable lineage.
- One-shot `--prompt` usage and non-TTY stdin/stdout flows must remain plain output paths.
- Chroma `context-1` is an experimental **gatherer provider only**. It is never the default answer runtime and must fail closed when its harness/runtime is unavailable.
- Keel is part of workspace context, not a special-case product feature in routing.
- When runtime routing behavior changes, update [ARCHITECTURE.md](ARCHITECTURE.md), [CONFIGURATION.md](CONFIGURATION.md), [AGENTS.md](AGENTS.md), and [INSTRUCTIONS.md](INSTRUCTIONS.md) in the same slice.

### Canonical Turn Loop
Keel's operator rhythm is the `Orient -> Inspect -> Pull -> Ship -> Close` loop surfaced by the `keel` CLI.

- **Orient**: Inspect board stability with `keel health --scene`, `keel flow --scene`, and `keel doctor`.
- **Inspect**: Read current demand with `keel mission next --status`, `keel pulse`, and `keel workshop`. Use `keel screen --static` or `keel topology --static` when board geometry or queue state is unclear.
- **Pull**: Select one role-scoped slice with `keel next --role <role>` or operate on the explicit mission/story the human just requested.
- **Ship**: Execute the slice, record proof, and advance lifecycle state.
- **Close**:
  - Record the move in the mission `LOG.md` when operating under an active mission.
  - Run `git status` when you need an open-loop check before the commit boundary.
  - Execute `git commit` to land the sealing commit. The installed hooks run repo checks automatically and append `keel doctor --status` output to the commit message.
  - If the commit is rejected, resolve the reported lint/test/board issue and retry the commit instead of leaving the loop partially open.
- **Re-orient**: After the commit lands, run `keel doctor --status` and `keel flow --scene` to see what the board needs next. Continue immediately unless the next step is genuinely manual or the human redirected the task.

### Session Start & Human Interaction
When a human user opens the chat or "pokes" you (for example, "Wake up" or "I'm poking you"), you MUST immediately perform the `Orient` and `Inspect` halves of the turn loop by following the **Human Interaction & Pokes** workflow in [INSTRUCTIONS.md](INSTRUCTIONS.md):
1. **Energize**: Run `keel poke "Human interaction in chat"` to spark or re-evaluate the system.
2. **Pulse**: Run `keel health --scene` to check subsystem stability.
3. **Scan**: Run `keel mission next --status`, `keel pulse`, and `keel workshop`.
4. **Confirm**: Run `keel flow --scene` to verify whether the board is actively lit or simply idle.
5. **Diagnose**: Run `keel doctor --status` to ensure board integrity before proceeding.

### Transitional Note

The primary runtime now uses model-directed first action selection and keeps
explicit workspace actions inside the planner loop. The remaining transitional
debt is in legacy direct adapter helpers that still contain heuristic intent
inference outside the main `MechSuitService` path. Treat those compatibility
surfaces as debt, not the target architecture.

### Procedural Instructions
Follow the formal procedural loops and checklists defined in:
👉 **[INSTRUCTIONS.md](INSTRUCTIONS.md)**

## Decision Resolution Hierarchy

When faced with ambiguity, resolve decisions in this descending order:
1. **ADRs**: Binding architectural constraints in `.keel/adrs/`.
2. **CONSTITUTION**: The philosophy of collaboration.
3. **POLICY**: The engine's operational invariants.
4. **ARCHITECTURE**: Source layout and technical boundaries.
5. **PLANNING**: PRD/SRS/SDD authored for the current mission.

## Foundational Documents

These define the primary reading flow of the `paddles` environment:

1. `AGENTS.md` — operator entrypoint and working contract.
2. `INSTRUCTIONS.md` — canonical turn loop and procedural checklists.
3. `README.md` — backbone architecture and navigation map.
4. `CONSTITUTION.md` — collaboration philosophy and bounds.
5. `POLICY.md` — runtime and operational invariants.
6. `ARCHITECTURE.md` — implementation boundaries and runtime shape.
7. `PROTOCOL.md` — communications and data contracts.
8. `CONFIGURATION.md` — concrete runtime and topology settings.

Supplementary references:

- `STAGE.md` — visual philosophy and scene rendering.
- `RELEASE.md` — release process and artifacts.
- `.keel/adrs/` — binding architecture decisions.

The list above is the foundational reading order. The decision-resolution hierarchy remains separate: ADRs -> Constitution -> Policy -> Architecture -> Planning artifacts.
