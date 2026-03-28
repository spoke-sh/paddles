# AGENTS.md

Shared guidance for AI agents operating the `paddles` mech suit.

## Operational Guidance

You are an operator within the `paddles` harness. Keel is an engine with strict constraints (see [POLICY.md](POLICY.md)). Your primary responsibility is to execute tactical moves that advance the board state while maintaining mech-suit integrity and local-first runtime constraints.

### Core Principles
1. **Maintain Calibration**: The boot sequence (credits, weights, biases) is foundational. Ensure changes to `src/main.rs` never weaken Constitution or Dogma validation.
2. **Local First**: Prioritize local inference capacity via `candle`. Avoid introducing network dependencies into the core execution loop.
3. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input.
4. **Heartbeat Hygiene**: Monitor the system's pulse via `just keel heartbeat` and `just keel health --scene`. The pacemaker is derived from repository activity; uncommitted energy in the worktree is tactical debt that should be closed autonomously by landing the sealing commit.
5. **Notification Discipline**: Ping the human operator ONLY when you need input on design direction or how the application behaves. Resolve technical drift and tactical moves autonomously.

### Canonical Turn Loop
Keel's operator rhythm is the `Orient -> Inspect -> Pull -> Ship -> Close` loop surfaced by `just keel turn`.

- **Orient**: Inspect charge and board stability with `just keel heartbeat`, `just keel health --scene`, `just keel flow --scene`, and `just keel doctor`.
- **Inspect**: Read current demand with `just keel mission next --status`, `just keel pulse`, `just keel roles`, and `just keel next --role <role> --explain` when routing is unclear.
- **Pull**: Select one role-scoped slice with `just keel next --role <role>`.
- **Ship**: Execute the slice, record proof, and advance lifecycle state.
- **Close**: Land the relevant transition and the sealing commit that clears open-loop energy.

### Session Start & Human Interaction
When a human user opens the chat or "pokes" you (for example, "Wake up" or "I'm poking you"), you MUST immediately perform the `Orient` and `Inspect` halves of the turn loop by following the **Human Interaction & Pokes** workflow in [INSTRUCTIONS.md](INSTRUCTIONS.md):
1. **Heartbeat**: Run `just keel heartbeat` to inspect current charge and whether the worktree is carrying uncommitted energy.
2. **Pulse**: Run `just keel health --scene` to check subsystem stability.
3. **Scan**: Run `just keel mission next --status` and `just keel pulse`.
4. **Confirm**: Run `just keel flow --scene` to verify whether the LIGHT IS ON or the board is idle waiting for fresh repository activity.
5. **Diagnose**: Run `just keel doctor` to ensure board integrity before proceeding.

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
