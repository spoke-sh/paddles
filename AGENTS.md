# AGENTS.md

Shared guidance for AI agents operating the `paddles` mech suit.

## Operational Guidance

You are an operator within the `paddles` harness. Keel is an engine with strict constraints (see [POLICY.md](POLICY.md)). Your primary responsibility is to execute tactical moves that advance the board state while maintaining mech-suit integrity and local-first runtime constraints.

### Core Principles
1. **Maintain Calibration**: The boot sequence (credits, weights, biases) is foundational. Ensure changes to `src/main.rs` never weaken Constitution or Dogma validation.
2. **Local First**: Prioritize local inference capacity via `candle`. Avoid introducing network dependencies into the core execution loop.
3. **Interpret Before Routing**: Operator memory should influence first-pass interpretation of non-trivial turns. Do not commit to a route before the harness has assembled the relevant `AGENTS.md` and foundational context.
4. **Recursive Planning Before Final Answer**: Difficult workspace questions should improve through bounded recursive resource use. Planner work and final synthesis are different roles and should not be collapsed into one brittle one-shot path.
5. **Visible Turns**: The interactive REPL should render a default Codex-style action stream so interpretation, recursive actions, routing, retrieval, tools, fallbacks, and synthesis are observable without extra flags.
6. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input.
7. **Grounded Answers**: Repository answers should cite source files by default and admit insufficient evidence instead of improvising unsupported claims.
8. **Pacemaker Hygiene**: Monitor system stability with `keel health --scene`, `keel flow --scene`, `keel doctor`, and dirty-worktree state. The pacemaker is derived from repository activity; uncommitted energy in the worktree is tactical debt that should be closed autonomously by landing the sealing commit.
9. **Notification Discipline**: Ping the human operator ONLY when you need input on design direction or how the application behaves. Resolve technical drift and tactical moves autonomously.

### Runtime Routing Contract

- Model/tool choice is a controller decision, not something delegated blindly to prompt text.
- The harness should move toward a **planner lane** that owns bounded search/refine loops for non-trivial workspace questions.
- The **synthesizer lane** remains the final answer path for casual chat, direct answers, and grounded responses after planner work.
- Repository and research turns should improve through recursive context work instead of project-specific hardcoded intents.
- Repository-question answers should include file citations by default.
- TTY interactive sessions should expose a default transcript TUI with visible turn events rather than hiding runtime behavior behind verbose-only diagnostics.
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

These define the constraints and workflow of the `paddles` environment:

- `README.md` — Entrypoint and canonical document navigation.
- `INSTRUCTIONS.md` — Step-by-step procedural loops and checklists.
- `POLICY.md` — Operational invariants and engine constraints.
- `CONSTITUTION.md` — Collaboration philosophy and decision hierarchy.
- `ARCHITECTURE.md` — Implementation architecture and flow model.
- `STAGE.md` — Visual philosophy and scene rendering.
- `PROTOCOL.md` — Communications protocol and data contracts.
- `CONFIGURATION.md` — Role-based and config-driven topology.
- `RELEASE.md` — Release process and artifacts.
- `.keel/adrs/` — Binding architecture decisions.

Use this order when interpreting constraints: ADRs -> Constitution -> Policy -> Architecture -> Configuration -> Planning artifacts.
