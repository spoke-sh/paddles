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
paddles --model qwen-1.5b --planner-model qwen3.5-2b --gatherer-provider sift-direct
```

Provider selection is now lane-specific as well. The synthesizer lane uses
`--provider <name>`, while the planner can either inherit that provider or set
its own with `--planner-provider <name>`:

```bash
paddles \
  --provider openai --model gpt-4o \
  --planner-provider anthropic --planner-model claude-sonnet-4-20250514 \
  --gatherer-provider sift-direct
```

That keeps providers authenticated side-by-side and lets the planner/synthesizer
mix providers without restarting the session.

If you want a distinct local gatherer model instead of the direct retrieval backend:

```bash
paddles --model qwen-1.5b --gatherer-model qwen-coder-1.5b --gatherer-provider local
```

For local sift-backed retrieval, select the explicit provider:

```bash
paddles --model qwen-1.5b --gatherer-provider sift-direct
```

That backend stays local-first and services bounded planner search.

- `paddles` remains the only recursive planner in the runtime path.
- Direct retrieval progress is surfaced as concrete execution stages instead of autonomous planner states.
- The provider returns evidence bundles and retrieval metadata consumed by the planner and synthesizer lanes.
- The legacy config value `sift-autonomous` is accepted as a compatibility alias and normalized to `sift-direct`.

See [SEARCH.md](SEARCH.md) for the full search boundary and capability contract.

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

### Provider Credentials

Remote HTTP providers resolve API credentials in this order:

1. provider-specific environment variable
2. `~/.config/paddles/credentials.toml`

The credential manifest stores only provider-to-file references. Each secret is
written to its own file under `~/.config/paddles/keys/`, for example:

```toml
[keys]
moonshot = "keys/moonshot.key"
openai = "keys/openai.key"
```

Key files are written with restricted permissions on Unix systems. In the TUI,
use `/login <provider>` to enter a provider API key through masked input;
`paddles` updates the referenced key file and rebuilds the active runtime lanes
so subsequent turns use the new credential immediately. `/model` shows the full
known model catalog; providers with resolved credentials are marked enabled and
providers without credentials are marked disabled. The local `sift` provider
does not use API-key login.

Successful `/model` changes are written to the machine-managed runtime lane
state file at `~/.local/state/paddles/runtime-lanes.toml`. That file preserves
the last selected planner/synthesizer lanes across restarts without mutating
authored `paddles.toml` files. Runtime lane state is applied after authored
config for the planner/synthesizer lane fields, so the last `/model` selection
is restored even when a project-local `./paddles.toml` also sets `provider` or
`model`. Other settings such as `port`, verbosity, gatherer configuration, and
CLI flags keep their normal precedence.

For OpenAI-compatible remote providers, planner turns now use native tool calls
to select the next bounded workspace action. The provider chooses the action
through the planner tool, and Paddles executes that action locally in the
workspace harness.

For Moonshot, the current API model id is `kimi-k2.5`. Legacy configs using
`kimi-2.5` are normalized to `kimi-k2.5` at runtime for compatibility.

For Inception, the supported core model path is `mercury-2`. Authenticate with
`/login inception`, then select it with `/model synthesizer inception mercury-2`
or `/model planner inception mercury-2`. That chat-completions compatibility
path is usable today without provider-native streaming/diffusion views. For
single-file `apply_patch` workspace actions on an Inception-backed lane,
Paddles now uses the provider-native `mercury-edit` companion endpoint behind
the scenes.

For the local `sift` provider, `bonsai-8b` is now available as an opt-in local
model path:

```bash
paddles --provider sift --model bonsai-8b
```

That path uses the stable `sift` local model boundary, but Bonsai currently
loads from Prism's official unpacked safetensors bundle rather than the GGUF
release. The published 1-bit GGUF artifact is not yet executable through the
upstream `metamorph` -> Candle compatibility path for this model, so paddles
uses the unpacked compatibility bundle instead. This is still a compatibility
path and does not preserve the original 1-bit runtime efficiency.

### Final Answer Render Capability

Paddles resolves final-answer rendering capability at boot from the selected
provider/model pair and then uses the strictest supported transport for the
synthesizer lane:

- `openai`: native JSON Schema via Chat Completions `response_format`
- `anthropic`: native structured tool use with a forced render tool
- `google`: native JSON Schema via Gemini `generationConfig`
- `inception`: OpenAI-compatible JSON Schema via the `mercury-2` chat completions path
- `moonshot`, `ollama`, and local `sift`: prompt-enveloped JSON with post-response normalization

All providers still normalize into the same transcript-safe render envelope
(`paragraph`, `bullet_list`, `code_block`, `citations`), and slightly
inconsistent envelopes are repaired from the emitted `blocks` rather than shown
raw.

### Inception Capability Boundary

The supported Inception boundary is now:

- Core chat compatibility: `mercury-2` through the existing OpenAI-compatible chat adapter, including structured final answers and forensic capture.
- Native edit companion: single-file `apply_patch` mutations on an Inception-backed lane use `v1/apply/completions` with the `mercury-edit` companion model.
- Optional native capabilities: provider-specific streaming/diffusion behavior remains a follow-on slice.
- Operator expectation: Inception is usable today for planner or synthesizer lanes, and patch-style mutation actions can use the native edit companion without requiring a separate visible model lane.

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
