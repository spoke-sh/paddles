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

`paddles` now treats runtime configuration as shared/planner/synth/gatherer
lane selection rather than single-model-only routing:

- The **shared lane selection** is the default provider/model pair for both planner and synthesizer lanes.
- The **synthesizer lane** is the default response path and must always be configured.
- The **planner lane** owns first bounded action selection for the primary mech-suit path: `answer` / concrete workspace actions (`search`, `list_files`, `read`, `inspect`, `shell`, `diff`, `write_file`, `replace_in_file`, `apply_patch`) / `refine` / `branch` / `stop`.
- The **gatherer backend** services planner search/refine actions when workspace retrieval is needed, including optional structural fuzzy retriever overrides selected by the planner.
- If the planner or gatherer backend is unavailable, `paddles` emits labeled fallback events and degrades honestly to the remaining local-first path.

`paddles.toml` can define those model selections explicitly:

```toml
[shared]
provider = "openai"
model = "gpt-4o"

[synthesizer]
model = "gpt-4o-mini"

[planner]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
```

When `[synthesizer]` or `[planner]` is omitted, that lane falls back to
`[shared]`. CLI flags still win over everything:

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
- Planner `search` and `refine` actions may now add `retrievers=["path-fuzzy"]` or `retrievers=["path-fuzzy","segment-fuzzy"]` when `sift` should bias toward structural fuzzy lookup.
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
- automatic plan updates when planned turns need explicit execution containment
- planner action selection
- gatherer capability and gathered evidence
- planner summaries and stop reasons
- tool calls, live background terminal stdout/stderr, and final results
- fallback reasons
- grounded or insufficient-evidence synthesis

Lower-verbosity sessions intentionally keep direct-response bookkeeping terse.
For casual turns, the first planner step remains visible, but route
classification, route-selection bookkeeping, and "synthesized direct answer"
status rows stay behind higher verbosity tiers.
The default TUI stream also folds matching planner-owned `inspect` calls into
their planner-step row and suppresses duplicate tooling/governor shadow rows
when the underlying tool output or fallback is already visible.

### Verbosity Levels

Paddles resolves one shared runtime verbosity level and applies it to both the
TUI transcript stream and the web UI event stream.

- `0` (`default`) keeps the stream operational: interpretation, first planner
  action, plan updates, tool/gatherer work, fallbacks, grounded synthesis, and
  insufficient-evidence outcomes remain visible. Direct-response bookkeeping is
  intentionally suppressed at this level.
- `1` (`-v`) adds the next layer of control metadata: direct-answer synthesis
  status, planner summaries and stop reasons, context strain/refinement
  summaries, and other info-tier turn events.
- `2` (`-vv`) adds debug-tier routing detail: route classification and route
  selection rows, provider capability checks, richer interpretation rendering,
  and adapter debug summaries such as HTTP request/response envelopes.
- `3` (`-vvv`) enables trace-tier diagnostics: boot sequencing, full prompt
  payload tracing, and the most verbose local/remote model transport logs.

In the TUI, slow steps can promote some event rows into view to keep long turns
legible, but low-value direct-response bookkeeping still stays quiet unless the
resolved verbosity explicitly allows it.

Repository-question answers also include source/file citations by default.
When a turn carries edit pressure, grounding pressure, or multi-step follow-up
pressure from recent conversation turns, Paddles now emits a Codex-style
checklist of the remaining work in the stream and feeds the same unfinished
items back into planner loop notes so the execution stays contained until the
checklist is complete.
Known-edit turns also get one bounded replan when they hit planner budget
before an applied edit lands. That replan keeps the current evidence, updates
the checklist, and expands the per-turn read/inspect/search envelope instead of
dropping straight to the blocked-edit reply.
Prompted git-commit turns now use the same containment path. When the prompt
explicitly asks Paddles to record a commit, the controller opens a commit
obligation, bootstraps a `git status --short` probe if the planner tried to
answer directly, and keeps the turn open until a `git commit` shell action
lands. After `git status` and `git diff` have already been inspected,
action-bias review steers advice-only `stop` answers back toward recording the
commit instead of ending the turn with guidance text.

### Deterministic Edit Target Resolution

Edit-oriented turns now run through a deterministic authored-path resolver
before broad search churn or workspace mutation continues. The resolver
self-discovers repository paths, respects the root `.gitignore` boundary when
present, and treats generated/vendored trees as out of scope unless no authored
boundary is available.

Resolver outcomes are explicit:

- `resolved` means one authored path won the ranking and can be promoted into
  read/diff/edit actions.
- `ambiguous` means multiple authored candidates remained tied, so the turn must
  narrow further before mutating the workspace.
- `missing` means no safe authored target matched the hint, so mutation fails
  closed and the controller replans instead of guessing.

Those outcomes are recorded into the trace and projected into the web
manifold/forensic views, so operators can see why a target advanced, stalled,
or was blocked. This feature is intentionally not a full semantic IDE/LSP
system: it resolves authored workspace paths deterministically, but it does not
depend on editor state or claim full symbol intelligence.

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
the last selected shared runtime model across restarts without mutating
authored `paddles.toml` files. Runtime lane state is applied after authored
config for the model lane fields, so the last `/model` selection is restored
even when a project-local `./paddles.toml` also sets shared/planner/synthesizer
models. When older runtime lane state still points at OpenAI Responses-only
`*-pro` variants such as `gpt-5.4-pro`, Paddles rewrites that machine-managed
state onto the corresponding chat-completions model during startup, and any
older machine-managed planner split is dropped so `/model` remains a single
shared-lane override. Other settings such as `port`, verbosity, gatherer
configuration, and CLI flags keep their normal precedence. If no authored config layer explicitly
sets `port` and you do not pass `--port`, Paddles asks the OS for an ephemeral
HTTP port at startup and reports the resolved HTTP bind address in the intro
transcript. The primary web routes prefer the built `apps/web/dist` runtime
when it is present, but installed release builds fall back to the compiled-in
web shell embedded in the Rust binary so the web UI still renders when no local
frontend dist tree exists beside the executable. Verbosity follows the same
precedence order too: CLI `-v`, `-vv`, or `-vvv` wins over authored
`paddles.toml`, and the resolved level applies to both the TUI transcript
stream and the web UI event stream. When neither the CLI nor authored config
sets verbosity, Paddles uses level `0`.

### Embedded Fallback Shell Parity Boundary

The shipped fallback artifact is the compiled-in `src/infrastructure/web/index.html` shell.
It is intentionally a single-file DOM/JS runtime rather than a mirror of the
decomposed React source tree under `apps/web/src`.

That fallback must preserve the core operator contract for the primary routes,
chat transcript/composer, live stream rows, tool output, plan updates,
forensic inspector, transit trace, manifold route, transcript-driven manifold
turn selection, and sticky-tail chat scrolling. It does not need React
component/module parity as long as those operator-facing behaviors remain
aligned and the bounded differences stay documented.

Local `sift` retrieval artifacts are machine-managed too. Paddles stores the
search cache under `~/.cache/paddles/sift/workspaces/<workspace-key>` instead
of inside the repository workspace, and the repo-level `.siftignore` excludes
workspace-local `.sift/**` paths so search never indexes its own cache tree.

For OpenAI-compatible remote providers, planner turns now use native tool calls
to select the next bounded workspace action. The provider chooses the action
through the planner tool, and Paddles executes that action locally in the
workspace harness.

For OpenAI specifically, the current remote transport stays on Chat Completions
with structured JSON/tool calls. Responses-only OpenAI models such as
`gpt-5.4-pro`, `gpt-5-pro`, and `gpt-5.2-pro` are rejected up front; use
`gpt-5.4`, `gpt-5.4-mini`, or `gpt-4o` instead.

For Moonshot, the current API model id is `kimi-k2.5`. Legacy configs using
`kimi-2.5` are normalized to `kimi-k2.5` at runtime for compatibility.

For Inception, the supported core model path is `mercury-2`. Authenticate with
`/login inception`, then select it with `/model inception mercury-2`. That chat-completions compatibility
path is usable today without provider-native streaming/diffusion views.
Workspace edits still execute locally through the shared workspace editor
boundary, even when the planner lane is Inception-backed.

For the local `sift` provider, `bonsai-8b` is now available as an opt-in local
model path:

```bash
paddles --provider sift --model bonsai-8b
```

That path uses the stable `sift` local model boundary. Bonsai now resolves from
Prism's published GGUF source through the upstream `metamorph` -> Candle
compatibility path rather than a direct unpacked-bundle download. This is still
a compatibility path and does not preserve the original 1-bit runtime
efficiency.

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
- Local workspace editor: `write_file`, `replace_in_file`, `diff`, and `apply_patch` execute through one provider-agnostic local workspace editor boundary, but that boundary is limited to authored workspace files. When the workspace has a root `.gitignore`, Paddles uses it as the primary authored-file boundary for planner targeting, `list_files`, gatherer evidence filtering, and execution-time edit rejection. Only when no root `.gitignore` is present does Paddles fall back to a small generated/vendored directory denylist (`node_modules`, `dist`, `result`, `target`, `.docusaurus`, `.turbo`, `.sift`, `.direnv`, plus `.git`).
- Optional native capabilities: provider-specific streaming/diffusion behavior remains a follow-on slice.
- Operator expectation: Inception is usable today for planner or synthesizer lanes without changing local workspace edit semantics.

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
