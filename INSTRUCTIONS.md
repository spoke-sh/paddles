# INSTRUCTIONS.md

Procedural instructions and workflow guidance for agents and operators working with the `paddles` mech suit.

## The Turn Loop

Keel is an engine with strict constraints. Your job is to move the board through the canonical `Orient -> Inspect -> Pull -> Ship -> Close` loop while eliminating drift.

The `keel` CLI is the canonical reference surface for this rhythm. Every session follows this deterministic cycle:

1. **Orient**: Run `keel health --scene`, `keel flow --scene`, and `keel doctor`. This tells you whether the board is healthy, structurally coherent, and ready for work.
2. **Inspect**: Run `keel mission next --status`, `keel pulse`, and `keel workshop`. If routing or board geometry is unclear, inspect `keel screen --static` or `keel topology --static`.
3. **Pull**: Choose the correct lane and role (`manager`, `operator`, or a configured role family) and pull exactly ONE slice of work.
4. **Ship**: Execute the move, record proof while the work is fresh, and land the relevant lifecycle transition (`story submit`, `voyage plan`, `bearing assess`, and so on).
5. **Close**:
   - Record your move in the mission `LOG.md` when operating under an active mission.
   - **Open-Loop Check**: Use `git status` if you need to inspect current worktree energy before the commit boundary.
   - **Commit**: Execute `git commit`. The installed hooks automatically run repo checks and append `doctor --status` to the commit message. Resolve any issues if the commit is rejected.
6. **Re-orient**: After the commit lands, run `keel doctor --status` and `keel flow --scene` to see what the board needs next. This is the "plug the cord back in" moment. If the delivery lane has ready work, start the next turn immediately. Only stop to ask the human when you reach a manual lane (design direction, bearing assessment, or human verification) or when the user has explicitly redirected you.

## Primary Workflows

### Operator (Implementation)
Focus on **evidence-backed delivery**.
- **Context**: `keel story show <id>`, `keel voyage show <id>`, and `keel next --role operator`.
- **Action**: Implement requirements, record proofs with `keel story record`, run `keel verify run` when needed, and `keel story submit`.
- **Constraint**: Every AC must have a proof.

### Manager (Planning)
Focus on **strategic alignment and unblocking**.
- **Context**: `keel epic show <id>`, `keel mission show <id>`, `keel flow --scene`, and `keel screen --static`.
- **Action**: Author `PRD.md`, `SRS.md`, `SDD.md`, resolve routing, decompose stories, and attach mission children explicitly with `keel mission attach <mission-id> --epic <epic-id>`, `--bearing <bearing-id>`, or `--adr <adr-id>`.
- **Constraint**: Move voyages from `draft` to `planned` only when requirements are coherent.

### Explorer (Research)
Focus on **technical discovery and fog reduction**.
- **Context**: `keel bearing list` and `keel workshop`.
- **Action**: Fill `BRIEF.md`, collect `EVIDENCE.md`, and `keel bearing assess`.
- **Constraint**: Graduate to epics only when research is conclusive.

## Paddles Runtime Routing

Paddles treats inference as a routing problem, not a single-model problem.

1. **Controller Owns Routing**: Do not rely on the model to decide whether tools or context gathering are needed. The controller should route the turn.
2. **Smallest Capable Path First**: Prefer the cheapest local runtime that can satisfy the request without degrading behavior.
3. **Synthesizer Lane Default**: Casual chat, direct answers, and deterministic workspace/tool turns stay on the synthesizer lane.
4. **Gatherer Lane for Retrieval**: Retrieval-heavy or multi-hop turns may use the gatherer lane, but that lane must return typed evidence and capability state for a downstream synthesizer.
5. **Context-1 Boundary**: Chroma `context-1` is an experimental gatherer provider only. It must be selected explicitly, acknowledged with `--context1-harness-ready`, and fail closed when the harness/runtime is unavailable.
6. **Docs Move With Behavior**: Whenever routing heuristics, lane contracts, or provider boundaries change, update [ARCHITECTURE.md](ARCHITECTURE.md), [CONFIGURATION.md](CONFIGURATION.md), [AGENTS.md](AGENTS.md), and [INSTRUCTIONS.md](INSTRUCTIONS.md) in the same change slice.

## Human Interaction & Pokes

Keel's autonomous flow is governed by a physical battery metaphor, but the charge is now derived from real repository activity rather than a synthetic wake file.

If a human user pokes you (for example, "I'm poking you" or "Wake up"), you MUST:
1. **Energize**: Execute `keel poke "Human interaction in chat"` to spark or re-evaluate the system.
2. **Orient**: Run `keel health --scene`, `keel flow --scene`, and `keel doctor`.
3. **Inspect**: Run `keel mission next --status`, `keel pulse`, and `keel workshop` to identify any new work that has become ready or materialized.
4. **Route if Needed**: Use `keel screen --static` or `keel topology --static` when lane selection or board geometry needs clarification.
5. **Proceed**: If the board is idle but the human gave an explicit task, proceed with that task instead of blocking on the lack of queued work.

## Autonomous Backlog Discharge

As long as the system is **AUTONOMOUS (LIGHT ON)** and the circuit is healthy (no blown capacitors), you are responsible for discharging the delivery backlog during the `Pull` and `Ship` phases of the turn loop.

1. **Identify Ready Work**: Scan the delivery lane for stories in `backlog` that are not blocked by dependencies using `keel mission next --status`, `keel next --role operator`, and `keel flow --scene`.
2. **Autonomous Start**: For each ready story, execute `keel story start <id>`.
3. **Rube Goldberg Loop**: Transitioning a story to `in-progress` mutates the repository, which refreshes the derived heartbeat and keeps the circuit closed while you continue moving work.
4. **Priority**: Discharging the backlog is your primary tactical objective once energized. You must continue until the backlog is empty or the circuit trips.
5. **Loop Closure**: After every successful implementation or transition, you MUST land a sealing commit that captures the resulting board and code changes. This applies to ALL work, including storyless gardening or engine changes. The pacemaker warning is cleared by committing the dirty worktree, not by touching a synthetic heartbeat file.

## Global Hygiene Checklist

Apply these checks to **every change** before finalizing work:

1. **Doctor First**: `keel doctor` is the ultimate source of truth for board integrity. You MUST run the doctor at the start of every session. If the doctor reports errors or "Short Circuits", you MUST prioritize fixing those diagnostic orders before attempting any other work or architectural changes.
2. **The Health Loop**: Use `keel health --scene` for high-level triage. Subsystems are mapped as follows:
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
3. **Pacemaker Protocol**: The system's heartbeat is derived from Git/worktree activity. Inspect the worktree directly with `git status`, use `keel flow --scene` and `keel doctor --status` to understand board pressure, and clear open-loop energy by landing the sealing commit. The installed pre-commit hook keeps quality checks and tests tied to the commit boundary, and the commit-msg hook appends `doctor --status` to the message body.
4. **Gardening First**: You MUST tend to the garden (fixing `doctor` errors, discharging automated backlog, and resolving structural drift) BEFORE notifying the human operator or requesting input.
5. **Notification Threshold**: Only request human intervention when you reach a "Manual Lane" that requires design direction or a decision on application behavior (for example, assessing a Bearing, planning a Voyage, or human verification of a complex Story).
6. **Automated Guardrails**: You no longer need to run `just quality` or `just test` manually before every commit. The git hooks installed via `keel hooks install` automatically enforce these checks. If a commit fails, resolve the reported lints or test failures and try again.
7. **Lifecycle Before Commit**: Run board-mutating lifecycle commands before the atomic commit when they generate or rewrite `.keel` artifacts (for example `story submit`, `voyage plan`, `voyage done`, `bearing assess`, `mission attach`). After the transition, inspect `git status` and include the resulting `.keel` churn in the same commit.
8. **Atomic Commits**: Commit once per logical unit of work. Use [Conventional Commits](https://www.conventionalcommits.org/):
   - `feat:` (new feature)
   - `fix:` (bug fix)
   - `docs:` (documentation)
   - `refactor:` (code change, no behavior change)
   - `test:` (adding/updating tests)
   - `chore:` (build/tooling)
9. **Mission Loop Discipline**: For mission-driven work, return to the mission steward loop after every completed story, planning unit, or bearing instead of continuing ad hoc from the last worker context.
10. **Knowledge Quality Bar**: Prefer no new knowledge over low-signal knowledge. A new knowledge entry should be novel, reusable across stories, and materially reduce future drift; otherwise link existing knowledge or omit capture entirely.
11. **Config Completeness**: Whenever introducing a new property to the configuration struct (`keel::infrastructure::config::Config`), you MUST immediately update `keel config show` expectations and any local configuration documentation so the new setting is visible to operators.
12. **Routing Completeness**: Whenever you change model selection, gatherer capability states, or synthesizer/gatherer contracts, update the foundational runtime docs in the same slice so operators are not forced to reverse-engineer the behavior from code.

## Upgrading Keel

Keel is managed as a Nix flake input in this repository.

To upgrade Keel to the latest version:
1. **Update the Flake**: Run `nix flake update keel`. This updates `flake.lock` to the latest commit on Keel `main`.
2. **Verify the Board**: Run `keel doctor`. Upgrading Keel may introduce new validation rules or schema changes. Fix any reported diagnostic issues.
3. **Update Hooks**: Run `keel hooks install` to align the local git hooks with the upgraded Keel version.
4. **Sync Operator Docs**: Reconcile `AGENTS.md` and `INSTRUCTIONS.md` with the latest upstream Keel structure while preserving Paddles-specific runtime/model-routing guidance.
5. **Seal & Commit**: Land the resulting `flake.lock` and instruction changes in one commit. The installed hooks append `doctor --status`.

## Compatibility Policy (Hard Cutover)

At this stage of development, this repository uses a **hard cutover** policy by default.

1. **No Backward Compatibility by Default**: Do not add compatibility aliases, dual-write logic, soft-deprecated schema fields, or fallback parsing for legacy formats unless a story explicitly requires it.
2. **Replace, Don't Bridge**: When introducing a new canonical token, field, command behavior, or document contract, remove the old path in the same change slice.
3. **Fail Fast in Validation**: `keel doctor` and transition gates should treat legacy or unfilled scaffold patterns as hard failures when they violate the new contract.
4. **Single Canonical Path**: Keep one source of truth for rendering, parsing, and validation; avoid parallel implementations meant only to preserve old behavior.
5. **Migration Is Explicit Work**: If existing board artifacts need updates, handle that in a dedicated migration pass or story instead of embedding runtime compatibility logic.

## Commands

### Command execution model (DRY)

Use one path for each concern:

- `just ...` for repo/build/test workflows and local `paddles` launchers.
- `keel ...` for all board/workflow operations.

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

### `keel` board workflow commands

Run `keel --help` for the full command tree. The core commands you should rely on:

| Category | Commands |
|----------|----------|
| Orientation | `keel health --scene` `keel flow --scene` `keel doctor` `keel screen --static` |
| Inspection | `keel mission next [<id>] --status` `keel pulse` `keel workshop` `keel topology --static` |
| Communication | `keel poke ...` `keel ping ...` `keel inbox` `keel outbox` |
| Discovery | `keel bearing new <name>` `keel bearing research <id>` `keel bearing assess <id>` `keel bearing list` |
| Planning | `keel mission new "<title>"` `keel epic new <name> --problem <problem>` `keel voyage new <name> --epic <epic-id> --goal <goal>` |
| Execution | `keel next --role <taxonomy>` `keel story new "<title>" [--type <type>] [--epic <epic-id> [--voyage <voyage-id>]]` `keel verify run` |
| Board Ops | `keel generate` `keel config show` `keel mission show <id>` `keel mission attach <mission-id> --epic <epic-id>` `keel mission attach <mission-id> --bearing <bearing-id>` `keel mission attach <mission-id> --adr <adr-id>` `keel audit [id]` |
| Lifecycle | Story/voyage/epic transitions in the table below |

## Story and Milestone State Changes

Use CLI commands only; do not move `.keel` files manually.

| Action | Command |
|--------|---------|
| Start | `keel story start <id>` |
| Reflect | `keel story reflect <id>` |
| Submit | `keel story submit <id>` |
| Reject | `keel story reject <id> "reason"` |
| Accept | `keel story accept <id> --role manager` |
| Ice | `keel story ice <id>` |
| Thaw | `keel story thaw <id>` |
| Voyage done | `keel voyage done <id>` |
