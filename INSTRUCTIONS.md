# INSTRUCTIONS.md

Procedural instructions and workflow guidance for agents and operators working with the `paddles` mech suit.

## The Turn Loop

Keel is an engine with strict constraints. Your job is to move the board through the canonical `Orient -> Inspect -> Pull -> Ship -> Close` loop while eliminating drift.

`just keel turn` is the canonical reference surface for this rhythm. Every session follows this deterministic cycle:

1. **Orient**: Run `just keel heartbeat`, `just keel health --scene`, `just keel flow --scene`, and `just keel doctor`. This tells you whether the board is energized, healthy, and structurally coherent.
2. **Inspect**: Run `just keel mission next --status` and `just keel pulse`. If routing is unclear, inspect `just keel roles` or `just keel next --role <role> --explain`.
3. **Pull**: Choose the correct lane and role (`manager`, `operator`, or a configured role family) and pull exactly ONE slice of work.
4. **Ship**: Execute the move, record proof while the work is fresh, and land the relevant lifecycle transition (`story submit`, `voyage plan`, `bearing lay`, and so on).
5. **Close**:
   - Record your move in the mission `LOG.md`.
   - **Heartbeat Check**: Use `just keel heartbeat` if you need to inspect the current activity source or confirm the circuit is still energized before the commit boundary.
   - **Commit**: Execute `git commit`. The installed hooks automatically run `just quality`, `just test`, auto-poke the board, and append `doctor --status` to the commit message. Resolve any issues if the commit is rejected.
6. **Re-orient**: After the commit lands, run `just keel doctor --status` and `just keel flow` to see what the board needs next. This is the "plug the cord back in" moment. If the delivery lane has ready work, start the next turn immediately. Only stop to ask the human when you reach a manual lane (design direction, bearing assessment, or human verification).

## Primary Workflows

### Operator (Implementation)
Focus on **evidence-backed delivery**.
- **Context**: `just keel story show <id>`, `just keel voyage show <id>`, and `just keel next --role operator`.
- **Action**: Implement requirements, record proofs with `just keel story record`, and `submit`.
- **Constraint**: Every AC must have a proof.

### Manager (Planning)
Focus on **strategic alignment and unblocking**.
- **Context**: `just keel epic show <id>`, `just keel roles`, `just keel next --role manager --explain`, and `just keel flow`.
- **Action**: Author `PRD.md`, `SRS.md`, `SDD.md`, resolve routing, decompose stories, and attach mission children explicitly with `just keel mission attach <mission-id> --epic <epic-id>`, `--bearing <bearing-id>`, or `--adr <adr-id>`.
- **Constraint**: Move voyages from `draft` to `planned` only when requirements are coherent.

### Explorer (Research)
Focus on **technical discovery and fog reduction**.
- **Context**: `just keel bearing list`.
- **Action**: Fill `BRIEF.md`, collect `EVIDENCE.md`, and `assess`.
- **Constraint**: Graduate to epics only when research is conclusive.

## Human Interaction & Pokes

Keel's autonomous flow is governed by a physical battery metaphor, but the charge is now derived from real repository activity rather than a synthetic wake file.

If a human user pokes you (for example, "I'm poking you" or "Wake up"), you MUST:
1. **Orient**: Execute `just keel heartbeat`, `just keel health --scene`, `just keel flow --scene`, and `just keel doctor`.
2. **Inspect**: Run `just keel mission next --status` and `just keel pulse` to identify any new work that has become ready or materialized.
3. **Route if Needed**: Use `just keel roles` or `just keel next --role <role> --explain` when lane selection or queue behavior needs clarification.

## Autonomous Backlog Discharge

As long as the system is **AUTONOMOUS (LIGHT ON)** and the circuit is healthy (no blown capacitors), you are responsible for discharging the delivery backlog during the `Pull` and `Ship` phases of the turn loop.

1. **Identify Ready Work**: Scan the delivery lane for stories in `backlog` that are not blocked by dependencies.
2. **Autonomous Start**: For each ready story, execute `just keel story start <id>`.
3. **Rube Goldberg Loop**: Transitioning a story to `in-progress` mutates the repository, which refreshes the derived heartbeat and keeps the circuit closed while you continue moving work.
4. **Priority**: Discharging the backlog is your primary tactical objective once energized. You must continue until the backlog is empty or the circuit trips.
5. **Loop Closure**: After every successful implementation or transition, you MUST land a sealing commit that captures the resulting board and code changes. This applies to ALL work, including storyless gardening or engine changes. The pacemaker warning is cleared by committing the dirty worktree, not by touching a synthetic heartbeat file.

## Global Hygiene Checklist

Apply these checks to **every change** before finalizing work:

1. **Doctor First**: `just keel doctor` is the ultimate source of truth for board integrity. You MUST run the doctor at the start of every session. If the doctor reports errors or "Short Circuits", you MUST prioritize fixing those diagnostic orders before attempting any other work or architectural changes.
2. **The Health Loop**: Use `just keel health --scene` for high-level triage. Subsystems are mapped as follows:
   - **NEURAL**: Stories (ID consistency, AC completion)
   - **MOTOR**: Voyages (Structure, SRS/SDD authorship)
   - **STRATEGIC**: Epics (PRD, Goal lineage)
   - **SENSORY**: Bearings (Research, Evidence quality)
   - **SKELETAL**: ADRs (Architecture decisions)
   - **VITAL**: Missions (Strategic achievement)
   - **AUTONOMIC**: Routines (Cadence, materialization)
   - **CIRCULATORY**: Workflow (Graph integrity, topology)
   - **PACEMAKER**: Heartbeat (derived repository activity and open-loop warning state)
   - **KINETIC**: Delivery (Backlog liquidity, execution capacity)
3. **Pacemaker Protocol**: The system's heartbeat is derived from Git/worktree activity and inspected with `just keel heartbeat`. A clean repo falls back to the latest commit; a dirty repo uses the freshest changed path it can observe. `doctor` warns when the worktree carries uncommitted energy, and the sealing commit is what clears that warning. The installed pre-commit hook keeps quality checks and tests tied to the commit boundary, and the commit-msg hook appends `doctor --status` to the message body.
4. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input.
5. **Notification Threshold**: Only request human intervention when you reach a "Manual Lane" that requires design direction or a decision on application behavior (for example, assessing a Bearing, planning a Voyage, or human verification of a complex Story).
6. **Automated Guardrails**: You no longer need to run `just quality` or `just test` manually before every commit. The git hooks installed via `just keel hooks install` automatically enforce these checks. If a commit fails, resolve the reported lints or test failures and try again.
7. **Lifecycle Before Commit**: Run board-mutating lifecycle commands before the atomic commit when they generate or rewrite `.keel` artifacts (for example `story submit`, `voyage plan`, `voyage done`, `bearing assess`, `bearing lay`). After the transition, inspect `git status` and include the resulting `.keel` churn in the same commit.
8. **Atomic Commits**: Commit once per logical unit of work. Use [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` (new feature)
   - `fix:` (bug fix)
   - `docs:` (documentation)
   - `refactor:` (code change, no behavior change)
   - `test:` (adding/updating tests)
   - `chore:` (build/tooling)
9. **Mission Loop Discipline**: For mission-driven work, return to the mission steward loop after every completed story, planning unit, or bearing instead of continuing ad hoc from the last worker context.
10. **Knowledge Quality Bar**: Prefer no new knowledge over low-signal knowledge. A new knowledge entry should be novel, reusable across stories, and materially reduce future drift; otherwise link existing knowledge or omit capture entirely.
11. **Config Completeness**: Whenever introducing a new property to the configuration struct (`keel::infrastructure::config::Config`), you MUST immediately update `just keel config show` expectations and any local configuration documentation so the new setting is visible to operators.

## Upgrading Keel

Keel is managed as a Nix flake input in this repository.

To upgrade Keel to the latest version:
1. **Update the Flake**: Run `nix flake update keel`. This updates `flake.lock` to the latest commit on Keel `main`.
2. **Verify the Board**: Run `just keel doctor`. Upgrading Keel may introduce new validation rules or schema changes. Fix any reported diagnostic issues.
3. **Update Hooks**: Run `just keel hooks install` to align the local git hooks with the upgraded Keel version.
4. **Seal & Commit**: Land the resulting `flake.lock` and instruction changes in one commit. The installed hooks will auto-poke the board and append `doctor --status`.

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a **hard cutover** policy by default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases, dual-write logic, soft-deprecated schema fields, or fallback parsing for legacy formats unless a story explicitly requires it.
2. **Replace, Don't Bridge**: When introducing a new canonical token, field, command behavior, or document contract, remove the old path in the same change slice.
3. **Fail Fast in Validation**: `just keel doctor` and transition gates should treat legacy or unfilled scaffold patterns as hard failures when they violate the new contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing, and validation; avoid parallel implementations meant only to preserve old behavior.
5. **Migration Is Explicit Work**: If existing board artifacts need updates, handle that in a dedicated migration pass or story instead of embedding runtime compatibility logic.

## Commands

### Command execution model (DRY)

Use one path for each concern:

- `just ...` for repo/build/test workflows.
- `just keel ...` for all board/workflow operations.

### `just` workflow commands

| Command | Purpose |
|---------|---------|
| `just` | List available recipes |
| `just build [profile]` | Build the project (`debug` by default, `release` optional) |
| `just build-release` | Build the release artifact |
| `just build-cuda` | Build the release artifact with CUDA support |
| `just test` | Run test suite (uses nextest) |
| `just quality` | Run formatting and clippy checks |
| `just paddles [--cuda ...]` | Run the `paddles` CLI |
| `just mission [--cuda]` | Run the standard verification path |

### `just keel` board workflow commands

Run `just keel --help` for the full command tree. The core commands you should rely on:

| Category | Commands |
|----------|----------|
| Orientation | `just keel turn` `just keel heartbeat` `just keel health --scene` `just keel flow --scene` `just keel doctor` |
| Inspection | `just keel mission next [<id>]` `just keel pulse` `just keel roles` `just keel next --role manager --explain` |
| Discovery | `just keel bearing new <name>` `just keel bearing research <id>` `just keel bearing assess <id>` `just keel bearing list` |
| Planning | `just keel epic new <name> --problem <problem>` `just keel voyage new <name> --epic <epic-id> --goal <goal>` |
| Execution | `just keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` |
| Board Ops | `just keel next --role manager` `just keel next --role operator` `just keel generate` `just keel config show` `just keel mission show <id>` `just keel mission attach <mission-id> --epic <epic-id>` `just keel mission attach <mission-id> --bearing <bearing-id>` `just keel mission attach <mission-id> --adr <adr-id>` |
| Lifecycle | Story/voyage/epic transitions in the table below |

## Story and Milestone State Changes

Use CLI commands only; do not move `.keel` files manually.

| Action | Command |
|--------|---------|
| Start | `just keel story start <id>` |
| Reflect | `just keel story reflect <id>` |
| Submit | `just keel story submit <id>` |
| Reject | `just keel story reject <id> "reason"` |
| Accept | `just keel story accept <id> --role manager` |
| Ice | `just keel story ice <id>` |
| Thaw | `just keel story thaw <id>` |
| Voyage done | `just keel voyage done <id>` |
