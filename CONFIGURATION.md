# Keel Configuration

Keel is configured via a `keel.toml` file. This document describes the available configuration options and how they drive Keel's behavior.

> Foundational stack position: `8/8`
> Read this after [PROTOCOL.md](PROTOCOL.md). This is the concrete settings reference at the end of the foundational stack.

## Configuration Resolution

Keel resolves configuration in the following order:

1.  **Project-local:** `./keel.toml`
2.  **User-global:** `~/.config/keel.toml`
3.  **Built-in defaults**

## Board Directory

You can specify where Keel stores its state (defaults to `.keel`):

```toml
board_dir = ".keel"
```

## Workflow & Topology

Keel uses a flexible, role-based lane topology to route work. This is configured via the `[workflow]`, `[roles]`, and `[lanes]` sections.

### Workflow Defaults

Defines the default roles and lanes used by commands like `keel next --role <role>` and `keel flow`.

```toml
[workflow.defaults]
management_role = "manager"
delivery_role = "operator"
management_lane = "management"
delivery_lane = "delivery"
```

### Lanes

Lanes are work queues that aggregate entities based on their status.

```toml
[lanes.delivery]
description = "Active delivery work"
include = ["story.backlog", "story.in-progress"]
exclude = ["story.done"]
parallel = true        # Allows parallel execution of independent stories
manual_accept = false  # Whether stories in this lane require manual verification
priority = 50          # Rendering and selection priority (higher first)
```

**Include/Exclude patterns:**
- `story.backlog`
- `story.in-progress`
- `story.*` (all story statuses)
- `voyage.draft`
- `bearing.exploring`

### Roles

Roles map a "role family" to a default lane and an operational contract.

```toml
[roles.operator]
default_lane = "delivery"
operational_contract = "operator-core"
```

**Operational Contract**: Refers to a built-in guidance profile that defines the actor's persona, priorities, and workflow hints. This is a binding agreement on how an actor must function when pulling work from the board. It is **not** a file-based markdown template.

Built-in contracts include:
- `manager-core`: Focused on planning, triage, and strategic alignment.
- `operator-core`: Focused on evidence-backed delivery and TDD.

### Role Overrides

You can provide specific overrides for full role taxonomy strings.

```toml
[role_overrides."operator/software:infrastructure"]
operational_contract = "infrastructure-operator"
```

## Scoring Modes

Keel uses scoring to prioritize work. You can switch between built-in modes or define custom ones.

```toml
[scoring]
mode = "constrained" # Options: constrained, growth, product, or custom
```

### Custom Modes

```toml
[scoring.modes.aggressive]
impact_weight = 3.0
confidence_weight = 2.0
effort_weight = 1.0
risk_weight = 0.5
```

## Doctor Diagnostics

Disable specific doctor checks if they don't apply to your workflow.

```toml
[doctor.checks.voyage-scope-authored-content]
disabled = true
```

## Research Providers

Configure weights and availability for research providers used in Bearings.

```toml
[research.providers.manual]
disabled = false
weight = 1.0

[research.providers.academic]
weight = 1.5
```

## Runtime Lane Selection

`paddles` now treats runtime configuration as planner/synth/gatherer lane
selection rather than single-model-only routing:

- The **synthesizer lane** is the default response path and must always be configured.
- The **planner lane** owns first bounded action selection for the primary mech-suit path: `answer` / concrete workspace actions (`search`, `list_files`, `read`, `inspect`, `shell`, `diff`, `write_file`, `replace_in_file`, `apply_patch`) / `refine` / `branch` / `stop`.
- The **gatherer backend** services planner search/refine actions when workspace retrieval is needed.
- If the planner or gatherer backend is unavailable, `paddles` emits labeled fallback events and degrades honestly to the remaining local-first path.

Today the lane wiring is exposed through CLI/runtime configuration:

```bash
paddles --model qwen-1.5b --planner-model qwen3.5-2b --gatherer-provider sift-autonomous
```

This keeps a light synthesizer while assigning a heavier planner to the recursive loop.

If you want a distinct local gatherer model instead of the autonomous backend:

```bash
paddles --model qwen-1.5b --gatherer-model qwen-coder-1.5b --gatherer-provider local
```

For local autonomous retrieval planning, select the explicit provider:

```bash
paddles --model qwen-1.5b --gatherer-provider sift-autonomous
```

That backend stays local-first and services bounded planner search.

- It defaults to the heuristic planner strategy.
- Recursive planner `search` / `refine` actions currently request bounded `graph` mode through the internal gatherer planning contract.
- The graph-mode trace is preserved as typed branch/frontier/node/edge metadata in the evidence bundle and default event stream.
- The internal contract still leaves room for future external artifact references, but large graph traces remain inline today.
- It returns planner trace metadata, stop reason, and retained artifact
  summaries inside the evidence bundle consumed by the synthesizer lane.
- The default REPL event stream surfaces planner action selection, gatherer
  summaries, and final planner stop reasons so operators can inspect the loop.

Current local model guidance on an 8 GB CUDA card:

- `qwen-1.5b` is the default Qwen2 instruct local path.
- `qwen-coder-0.5b` is the smaller fast coding fallback when latency matters more than capability.
- `qwen-coder-1.5b` remains available as the coding-tuned Qwen2 option.
- `qwen-coder-3b` remains available as the larger Qwen2 coding lane when you want more coding headroom.
- `qwen3.5-2b` remains available as an opt-in heavier lane, not the default.
- If `qwen3.5-2b` cannot complete its CUDA load or first generation step because of GPU OOM or a reduced-precision runtime mismatch, the runtime warns and retries on CPU instead of aborting the REPL.

### Default Turn Observability

The REPL now renders a default Codex-style action stream. Expect visible steps
for:

- interpretation context assembly
- model-selected first action
- resulting route classification
- route selection
- planner action selection
- gatherer capability and gathered evidence
- planner summaries and stop reasons
- tool calls and results
- fallback reasons
- synthesis readiness

Repository-question answers also include source/file citations by default.

### Experimental Context-1 Boundary

`context-1` is exposed as an explicit experimental gatherer provider, not as a
drop-in answer model:

```bash
paddles --model qwen-1.5b --gatherer-provider context1
```

That provider is explicit and honest about its readiness:

- Without `--context1-harness-ready`, the adapter reports `harness-required`
  and Paddles gracefully falls back to the synthesizer lane.
- With `--context1-harness-ready`, the adapter boundary reports the current
  harness state transparently until a real harness-backed implementation exists.
- The default REPL event stream surfaces the selected gatherer provider,
  capability state, fallback reason, and evidence summary — making diagnosis
  straightforward from terminal output.

## Subsystem Health (The Med-Bay)

The `keel health` command provides a high-level triage of the system's core subsystems. Use `--scene` for a visual bio-scan.

| Subsystem | Component | Description |
|-----------|-----------|-------------|
| **NEURAL** | Stories | Atomic implementation units and verification |
| **MOTOR** | Voyages | Tactical planning and SRS/SDD authorship |
| **STRATEGIC** | Epics | High-level initiatives and PRD coherence |
| **SENSORY** | Bearings | Research artifacts and evidence quality |
| **SKELETAL** | ADRs | Architecture Decision Records |
| **VITAL** | Missions | Core strategic objectives |
| **AUTONOMIC** | Routines | Scheduled automation and materialization |
| **CIRCULATORY** | Workflow | Board graph integrity and lane topology |
| **PACEMAKER** | Heartbeat | System energization and commit stability |

```bash
keel health --scene
```

## Full Example

```toml
board_dir = ".keel"

[workflow.defaults]
management_role = "manager"
delivery_role = "operator"

[roles.manager]
default_lane = "management"
template = "manager-core"

[roles.operator]
default_lane = "delivery"
template = "operator-core"

[lanes.management]
description = "Planning and verification"
include = ["bearing.*", "voyage.draft", "story.needs-human-verification"]
priority = 100
manual_accept = true

[lanes.delivery]
description = "Implementation"
include = ["story.backlog", "story.in-progress"]
parallel = true
priority = 50

[scoring]
mode = "constrained"

[doctor.checks.story-id-uniqueness]
disabled = false
```
## Trace Recording

The runtime recorder boundary is independent of transcript rendering:

- **Default runtime policy**: `noop` recorder — safe and local-first
- **Available local adapters**: in-memory and embedded `transit-core`
- **Growing edge**: a user-facing recorder-selection flag will land when the policy slice is ready

This keeps the live runtime local-first and safe while the recorder policy matures.
