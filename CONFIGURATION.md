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
model = "gpt-5.4"
thinking_mode = "high"

[synthesizer]
model = "gpt-4o-mini"

[planner]
provider = "anthropic"
model = "claude-sonnet-4-20250514"
```

`thinking_mode` is optional and currently only applies when the selected
provider/model catalog exposes selectable reasoning effort, provider-native
thinking toggles, or an explicit `none` mode.

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

### Harness Profiles

Paddles resolves an explicit harness profile from the prepared planner and synthesizer capability surfaces before each turn. The profile is not chosen by provider name. The current runtime contract is:

- `recursive-structured-v1`: active when planner next-action transport and final-answer rendering both stay structured
- `prompt-envelope-safe-v1`: explicit downgrade when prompt-envelope planner recovery or prompt-envelope rendering is required

The active profile owns:

- refinement policy for steering reviews
- bounded compaction budget
- recovery mode metadata for invalid model replies
- execution-governance posture for local hands, including sandbox mode, approval policy, and supported permission-reuse scopes
- active specialist-brain ids that may contribute bounded runtime notes

The selected profile and any downgrade reason are recorded on turn-start traces and reused in planner/gatherer metadata as the `profile` field.

Current execution-governance posture by profile:

- `recursive-structured-v1`: `workspace_write` sandbox, `on_request` approval, reuse scopes `turn`, `command_prefix`, and `hand`
- `prompt-envelope-safe-v1`: `workspace_write` sandbox, `on_request` approval, reuse scopes `turn` and `hand`; `command_prefix` reuse is explicitly disabled as part of the downgrade

The active posture is enforced through one shared permission gate for the
terminal runner and workspace editor. When a request exceeds the current
profile, the runtime returns a structured deny or escalation outcome instead of
silently broadening authority.

That selected posture is also projected into turn-time observability. Each turn
records a typed governance snapshot with the requested profile id, active
profile id, sandbox mode, approval policy, supported reuse scopes, and any
downgrade reason. Governed tool calls then record typed decision artifacts with
their permission request and allow, deny, or escalation outcome, so transcript,
trace, and API consumers can replay the exact governance story without parsing
free-form tool summaries.

External capability fabrics use the same projection discipline. Operator
surfaces should expect request and result rows to carry a stable vocabulary:

- `fabric` — the capability id such as `web.search`, `mcp.tool`, or `connector.app_action`
- `status` — the invocation outcome such as `succeeded`, `degraded`, `denied`, `unavailable`, or `failed`
- `availability` — the advertised fabric posture such as `available`, `stale`, `unauthenticated`, `disabled`, or `unavailable`
- `auth` — whether the fabric requires credentials
- `effects` — the side-effect posture (`read_only`, `potentially_mutating`, `mutating`)
- `provenance` — the surfaced source lineage for any returned external evidence

If a fabric is missing, stale, unauthenticated, or blocked by governance, the
runtime keeps those fields explicit in transcript, trace, and API payloads
instead of projecting a generic success string.

Current specialist-brain contract:

- `session-continuity-v1` is registered under both profiles so the planner contract stays stable across models
- the brain only activates when `recursive-structured-v1` is active and the session exposes durable turn summaries through `query_session_context(...)`
- unsupported profiles or empty session history produce an explicit runtime note rather than silently bypassing the recursive planner loop

Current local model guidance on an 8 GB CUDA card:

- `qwen-1.5b` is the default Qwen2 instruct local path.
- `qwen-coder-0.5b` is the smaller fast coding fallback when latency matters more than capability.
- `qwen-coder-1.5b` remains available as the coding-tuned Qwen2 option.
- `qwen-coder-3b` remains available as the larger Qwen2 coding lane when you want more coding headroom.
- `qwen3.5-2b` remains available as an opt-in heavier lane, not the default.
- If `qwen3.5-2b` cannot complete its CUDA load or first generation step because of GPU OOM or a reduced-precision runtime mismatch, the runtime warns and retries on CPU instead of aborting the REPL.

### Nix Build Lanes

The Nix flake now keeps CPU as the default lane:

- `nix develop` and `nix build .#paddles` stay CPU-only.
- `nix develop .#cuda`, `nix build .#paddles-cuda`, and `nix build .#sift-cuda` are the explicit GPU opt-in paths on Linux.
- `just paddles --cuda ...` still enables the Cargo `cuda` feature, but it should be run from the CUDA shell when GPU execution is actually intended.

That keeps CI and downstream consumers from pulling the CUDA toolchain unless the operator intentionally chooses the GPU lane.

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

At runtime, those provider credentials and any native-transport bearer tokens
are resolved through the shared `transport_mediator` hand instead of being read
ad hoc from generated shell/tool execution paths. The mediator also strips
known credential env vars from local terminal and workspace child processes
before they spawn.

Successful `/model` changes are written to the machine-managed runtime lane
state file at `~/.local/state/paddles/runtime-lanes.toml`. That file preserves
the last selected shared runtime model across restarts without mutating
authored `paddles.toml` files. Runtime lane state is applied after authored
config for the model lane fields, so the last `/model` selection is restored
even when a project-local `./paddles.toml` also sets shared/planner/synthesizer
models. When the selected shared model exposes provider-specific thinking
controls, the same machine-managed file also stores `thinking_mode = "..."`
next to the shared provider/model pair so `/model` restores that third-stage
selection on restart. When older runtime lane state still points at OpenAI Responses-only
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

For OpenAI specifically, non-reasoning turns on chat-completions models such as
`gpt-5.4`, `gpt-5.4-mini`, and `gpt-4o` stay on Chat Completions with
structured JSON/tool calls. When a GPT-5 lane enables thinking, Paddles now
routes planner tool calls and structured-answer requests through
`/v1/responses` so reasoning-aware turns stay compatible with OpenAI's
Responses-only tool/schema contract. Responses-only models such as
`gpt-5.4-pro`, `gpt-5-pro`, and `gpt-5.2-pro` continue to use
`/v1/responses` with reusable `previous_response_id` continuity.

For Moonshot, Paddles currently recognizes the Kimi API model ids
`kimi-k2.6`, `kimi-k2.5`, `kimi-k2`, `kimi-k2-0905-preview`,
`kimi-k2-0711-preview`, `kimi-k2-turbo-preview`, `kimi-k2-thinking`, and
`kimi-k2-thinking-turbo`. Legacy configs using `kimi-2.6` or `kimi-2.5` are
normalized to the corresponding `kimi-k2.*` ids at runtime for compatibility.

For Inception, the supported core model path is `mercury-2`. Authenticate with
`/login inception`, then select it with `/model inception mercury-2`. That chat-completions compatibility
path is usable today without provider-native streaming/diffusion views.
Workspace edits still execute locally through the shared workspace editor
boundary, even when the planner lane is Inception-backed.

### Negotiated Provider Capability Surface

Remote lane behavior now resolves from one negotiated capability surface per
provider/model pair instead of repeating provider-name branches in the runtime.
That surface currently carries five shared decisions:

- `http_format` — whether the lane uses the HTTP adapter and which wire format
  it speaks (`OpenAi`, `Anthropic`, or `Gemini`)
- `render_capability` — the strictest final-answer contract the synthesizer can
  use (`OpenAiJsonSchema`, `AnthropicToolUse`, `GeminiJsonSchema`, or prompt
  envelope fallback)
- `planner_tool_call` — how the remote planner selects its next bounded action
  (`NativeFunctionTool`, `StructuredJsonEnvelope`, or `PromptEnvelope`)
- `transport_support` — whether the selected model is supported on the current
  transport, plus an explicit operator-facing rejection reason when it is not
- `deliberation` — whether the provider exposes `native_continuation`,
  `summary_only`, `toggle_only`, or explicit `unsupported` no-op semantics, and
  whether any provider-native state can be round-tripped

When a future provider is added, extend this negotiated surface first. The
controller, planner, and renderer should consume the shared capability record
rather than branching on provider names for behavior that is conceptually the
same.

The provider catalog also publishes the selectable thinking modes for each
provider/model path. The CLI, TUI, and persisted runtime preferences validate
requested `thinking_mode` values against that catalog instead of relying on
provider-specific hard-coding.

Representative provider/model paths are documented below when one provider
family exposes more than one contract:

<!-- BEGIN_PROVIDER_CAPABILITY_MATRIX -->
| Provider | Model path | Wire | Support | Render | Planner | Deliberation | State | Thinking modes | Notes |
|---|---|---|---|---|---|---|---|---|---|
| `sift` | `qwen-1.5b` | `local` | `supported` | `prompt-envelope` | `prompt-envelope` | `unsupported` | `none` | `none` | Local native runtime; no provider-native reasoning substrate. |
| `openai` | `gpt-5.4` | `openai` | `supported` | `openai-json-schema` | `native-function-tool` | `toggle_only` | `none` | `none`, `low`, `medium`, `high`, `xhigh` | Chat Completions by default; thinking-enabled GPT-5 turns switch planner/schema requests to Responses. |
| `openai` | `gpt-5.4-pro` | `openai` | `supported` | `prompt-envelope` | `prompt-envelope` | `native_continuation` | `opaque_round_trip` | `none`, `low`, `medium`, `high`, `xhigh` | Responses path with reusable previous_response_id continuity. |
| `inception` | `mercury-2` | `openai` | `supported` | `openai-json-schema` | `native-function-tool` | `summary_only` | `none` | `instant`, `low`, `medium`, `high` | OpenAI-compatible chat with reasoning summaries but no reusable state. |
| `anthropic` | `claude-sonnet-4-20250514` | `anthropic` | `supported` | `anthropic-tool-use` | `prompt-envelope` | `native_continuation` | `opaque_round_trip` | `none`, `low`, `medium`, `high` | Messages API with thinking blocks and interleaved-thinking support. |
| `google` | `gemini-2.5-flash` | `gemini` | `supported` | `gemini-json-schema` | `structured-json-envelope` | `native_continuation` | `opaque_round_trip` | `none`, `low`, `medium`, `high` | generateContent path with thought-signature continuity. |
| `moonshot` | `kimi-k2.6` | `openai` | `supported` | `openai-json-schema` | `structured-json-envelope` | `native_continuation` | `opaque_round_trip` | `none`, `thinking` | OpenAI-compatible chat with reasoning_content continuity. |
| `ollama` | `qwen3` | `openai` | `supported` | `openai-json-schema` | `native-function-tool` | `toggle_only` | `none` | `none`, `thinking` | Freeform local models; qwen3 shown for thinking-capable toggle behavior. |
<!-- END_PROVIDER_CAPABILITY_MATRIX -->

- `summary_only` means the provider can surface reasoning summaries or similar
  observational output, but Paddles cannot round-trip reusable state.
- `toggle_only` means the provider exposes reasoning effort or visibility
  controls without reusable continuity state.
- `unsupported` means the harness treats the provider as an explicit no-op for
  provider-native reasoning continuity.

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

- `openai`: Chat Completions models use native JSON Schema via
  `response_format`; Responses-only models degrade to prompt-enveloped
  structured final answers over `/v1/responses`
- `anthropic`: native structured tool use with a forced render tool
- `google`: native JSON Schema via Gemini `generationConfig`
- `inception`: OpenAI-compatible JSON Schema via the `mercury-2` chat completions path
- `moonshot` and local `sift`: prompt-enveloped JSON with post-response normalization
- `ollama`: OpenAI-compatible JSON Schema on the current local HTTP boundary

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

### Shared Execution Hands

Paddles now names its local execution surfaces through one shared execution-hand
contract instead of leaving each adapter to invent local lifecycle labels. The
default session hand registry exposes three hands:

- `workspace_editor` — authored workspace mutation boundary
- `terminal_runner` — background shell execution boundary
- `transport_mediator` — credential-bearing transport and tool mediation boundary

Each hand reports the same lifecycle phases:

- `described`
- `provisioning`
- `ready`
- `executing`
- `recovering`
- `degraded`
- `failed`

Each hand also carries a stable authority label:

- `workspace_scoped` for local repo-bound execution surfaces
- `credential_mediated` for boundaries that own external credentials or live transport state

That vocabulary is session-scoped and provider-agnostic. Later workspace,
terminal, and transport adapters should update the shared hand diagnostics
surface rather than minting new state names for the same lifecycle transitions.

`GET /health` and `GET /session/shared/bootstrap` both expose this
`execution_hands` array beside `native_transports`, so operators can inspect
the current workspace editor, terminal runner, and transport mediator posture
from the same shared runtime surface they already use for transport readiness.

When native transport auth cannot resolve its configured `token_env`, or when a
required remote provider credential is missing, the `transport_mediator` hand
records that failure directly in `execution_hands` while the matching transport
or lane surfaces keep their own protocol-specific error detail.

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

## Native Transport Substrate

Authored transport enablement lives under one shared `native_transports` table:

```toml
[native_transports.http_request_response]
enabled = true
bind_target = "127.0.0.1:4100"

[native_transports.http_request_response.auth]
mode = "bearer_token"
token_env = "PADDLES_HTTP_TOKEN"

[native_transports.server_sent_events]
enabled = true
bind_target = "127.0.0.1:4100"

[native_transports.websocket]
enabled = false

[native_transports.transit]
enabled = false
```

Every transport slot resolves through the same shared diagnostics surface, regardless of protocol:

- `enabled` — whether the authored config has turned the transport on
- `phase` — one of `disabled`, `configured`, `binding`, `ready`, `degraded`, or `failed`
- `bind_target` — the address or endpoint the transport is trying to own
- `auth_mode` — currently `open` or `bearer_token`
- `last_error` — the latest bind, negotiation, or runtime failure recorded for the transport

`GET /health` and `GET /session/shared/bootstrap` both expose this `native_transports` array so operators can inspect transport readiness before transport-specific UIs or clients are added. Later protocol adapters should update the same diagnostics model instead of inventing side-channel health output.

### HTTP And SSE Operator Workflow

The first delivered native transport modes are `http_request_response` and `server_sent_events`.

Use `http_request_response` when the client only needs one-shot local request/response calls. Use `server_sent_events` when the client needs server-push streaming updates from the same runtime.

When both are enabled, they must share the same bind_target. The runtime hosts both modes on one listener, so authored HTTP and SSE `bind_target` values must either match exactly or leave one side unset so it can inherit the shared listener.

Operators should inspect these modes through the shared diagnostics endpoints:

- `GET /health` for a quick readiness snapshot
- `GET /session/shared/bootstrap` for the same diagnostics alongside the shared runtime projection

Interpret the shared diagnostics like this:

| Signal | Meaning |
|--------|---------|
| `phase = configured` | The transport is enabled in config but has not started binding yet |
| `phase = binding` | The runtime is trying to claim the configured listener |
| `phase = ready` | The listener is active and the transport is available on the shared web surface |
| `phase = failed` with `last_error` | Startup or runtime rejected the transport; inspect the error before retrying |

Common HTTP/SSE debugging rules:

- If both transports are enabled and `last_error` says they must share the same `bind_target`, align the authored targets first.
- If `phase = ready`, expect both transports to report the actual shared listener address in `bind_target`.
- If one of the two paths fails during startup, inspect both diagnostics rows because the shared listener setup can fail both together.

### WebSocket Session Adapter

The WebSocket transport upgrades on `GET /native-transports/websocket` when `[native_transports.websocket].enabled = true`.

It shares the same listener and diagnostics model as the HTTP and SSE paths:

- authored `bind_target` must align with any other enabled shared-web transports
- `phase = ready` means the listener is available for upgrades
- `session` is populated while a socket is actively established
- session-level protocol failures degrade the shared diagnostics row instead of disappearing into a socket-only side channel

The current adapter sends a `session_ready` frame when the socket upgrades, records the negotiated `connection_id` under the shared `session` diagnostic, and degrades the transport with `last_error` if the client sends a non-text prompt frame.

### Transit Structured Exchange Adapter

The Transit transport exchanges structured request/response payloads on `POST /native-transports/transit` when `[native_transports.transit].enabled = true`.

It also shares the same primary listener and diagnostics model as the HTTP, SSE, and WebSocket paths:

- authored `bind_target` must align with any other enabled shared-web transports
- `phase = ready` means the shared listener is available for Transit exchanges
- the adapter currently accepts `application/transit+json` request bodies only
- the current structured channel is `transit_exchange`
- negotiation or payload failures degrade the shared diagnostics row with `last_error`

The current adapter expects payloads shaped like:

```json
{
  "type": "turn_request",
  "channel": "transit_exchange",
  "prompt": "Inspect the workspace and summarize the CI failure."
}
```

Successful exchanges return `application/transit+json` with a `turn_response` payload. Negotiation failures return a `transport_error` payload instead of silently disappearing into HTTP-only status handling.

## Trace Recording

The runtime recorder boundary is independent of transcript rendering:

- **Default runtime policy**: embedded `transit-core` recorder rooted in machine-managed local state
- **Available local adapters**: embedded `transit-core` and bounded in-memory fallback
- **Shared session operations**: `wake`, full `replay`, `resume_from_checkpoint`, deterministic `replay_slice`, and higher-level `query_session_context`
- **Fallback posture**: if the persistent session spine cannot open, Paddles degrades to in-memory recording and emits an explicit boot warning
- **Growing edge**: a user-facing recorder-selection flag will land when the policy slice is ready

This keeps the live runtime local-first and safe while making durable session recording the normal path instead of optional metadata.

Authority selection is now explicit when you run non-interactive service mode:

```toml
[service_mode]
enabled = true
operator_surfaces_enabled = true

[trace_authority]
mode = "hosted_transit"
endpoint = "127.0.0.1:7171"
namespace = "prod"
service_identity = "paddles-primary"
```

- `service_mode.enabled = true` refuses implicit recorder fallback. You must
  choose a `trace_authority.mode`.
- Hosted service mode requires all three hosted fields:
  `endpoint`, `namespace`, and `service_identity`.
- Explicit local/dev fallback modes remain available as
  `mode = "embedded_local"` and `mode = "in_memory"`.
- When `service_mode.enabled = false`, the existing local-first default remains
  the embedded local Transit recorder rooted in machine-managed state.
- `service_mode.enabled = true` runs Paddles as a long-lived non-interactive
  service. It does not drop into the TUI or plain interactive loop.
- `service_mode.operator_surfaces_enabled = false` keeps hosted Transit as the
  primary runtime path and skips the HTTP/web operator surface entirely.

Hosted service mode emits an explicit runtime status snapshot on stdout so
operators can observe readiness or failure without attaching the TUI or the web
UI:

```json
{"mode":"service","state":"ready","authority_backend":"hosted_transit","authority_location":"127.0.0.1:7171#namespace=prod;service=paddles-primary","operator_surfaces":{"status":"disabled"}}
```

```json
{"mode":"service","state":"failed","authority_backend":"hosted_transit","authority_location":"127.0.0.1:7171#namespace=prod;service=paddles-primary","operator_surfaces":{"status":"disabled"},"failure":"..."}
```

Operator surfaces are reported as one of:

- `disabled`: hosted service mode is running without the HTTP/web operator surface
- `listening`: the optional operator surface is bound and serving
- `degraded`: the hosted authority is live, but the optional operator surface failed to bind

`query_session_context` is the stable harness-facing slice contract over the recorder:

- `AdaptiveReplay { turn_limit }` yields recent turn-bounded transcript context and turn summaries
- `Rewind { anchor, record_limit }` yields a deterministic backward slice from a task/turn/record/checkpoint/tail anchor
- `CompactionWindow { anchor, before_record_limit, after_record_limit }` yields a bounded neighborhood around an anchor for non-destructive compaction and replay inspection

The main runtime now asks the active session for an adaptive-replay slice before it falls back to persisted history or synthesizer-local summary caches. That keeps recent-turn handoff grounded in the durable session whenever recorder state is available.
