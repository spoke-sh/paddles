# Idiomatic Agent Vocabulary And Trusted Model Reasoning - Charter

Archetype: Strategic

## Mission Intent

Close the operator-experience gap between paddles and mature coding-agent
harnesses (Claude Code, Codex, Gemini CLI, OpenCode) by adopting the
vocabulary the field already uses, by trusting the planner model's own
reasoning end-to-end, and by streaming full tool output instead of clipping
it.

This is a refactor + UX mission. It must not introduce new "chambers,"
"specialist brains," or other bespoke abstractions. The product surface gets
smaller and more familiar; the planner/synthesizer split, the manifold trace
view, and the evidence-first thesis stay.

## Goals

| ID | Description | Verification |
|----|-------------|--------------|
| MG-01 | Stop overwriting the planner model's `decision.rationale` (currently at `src/application/recursive_control.rs:147-152`). Controller-derived signal summaries and governance notes must live on sibling fields, never on top of model-authored text. The forensic and manifold traces must show the model's own rationale verbatim. | board: VI2sGaOrg |
| MG-02 | Stream tool output (Shell, Inspect, Apply­Patch, semantic queries) to both the operator UI and the planner model as bytes arrive. Drop the `trim_for_planner(&rendered, 1_200)` cap in `src/application/planner_action_execution.rs`; raise any planner-bound budget to 32k+ with head+tail truncation and a marker. Raw output stays uncut in the trace recorder. | board: VI2sHovAf |
| MG-03 | Rename Tier-1 structural / metaphorical types and modules to industry vocabulary, behavior-preserving: `MechSuitService` → `AgentRuntime`; `*Chamber` → plain function modules (`agent_loop`, `context_assembly`, `synthesis`, `turn`); `RecursiveControlChamber` / `recursive_control` → `agent_loop` (ReAct loop); `ExecutionHand` → `ToolRunner`; `WorkspaceAction` / `WorkspaceActionExecutor` → `Tool` / `ToolExecutor`; `PlannerAction` → `AgentStep`; `instruction_frame` → `system_instructions`; `specialist_brains` → `subagents`; `harness_profile` → `runtime_profile`; `gatherer` → `retriever`; `forensics` → `trace` / `inspector`; `compaction_cue` → `compaction_trigger`; `premise_challenge` → `evidence_check`; `deliberation_signals` → `reasoning_signals`; `steering_signals` → `controller_signals`. Keep `transit` (separate library) and `manifold` (UI route) as-is. | board: VI2sJZcV9 |
| MG-04 | Promote `Collaboration::PlanOnly` (or equivalent) to a first-class `plan` mode in the TUI and CLI, mirroring Claude Code / Codex. Add `/help`, `/plan`, `/compact`, `/cost`, `/agents` (subagents), `/mcp` slash commands at minimum, alongside the existing `/login`, `/model`, `/resume`. | board: VI2sKP2cz |
| MG-05 | Prune docs (`README.md`, `ARCHITECTURE.md`, `CONSTITUTION.md`, `PROTOCOL.md`, `STAGE.md`, `POLICY.md`, `INSTRUCTIONS.md`, `CONFIGURATION.md`, `AGENTS.md`) to use the renamed vocabulary, and delete sections that describe non-existent capability ("automatic tier promotion," "concurrent sibling generation," "deterministic entity resolution" framed as deterministic, specialist brains beyond `session-continuity-v1`). Aspirational features either ship or get removed from the docs. | board: VI2sLV0uw |

## Constraints

- **No new bespoke abstractions.** No new `*Chamber`, no new `*Brain`, no new `*Hand`, no new `*Cue`, no new `Steering*`. If a new concept needs naming, use the term Claude Code / Codex / OpenCode would use.
- **Behavior-preserving renames.** MG-03 must land as mechanical rename PRs with no functional change. No "while we're here" refactors during a rename.
- **Trust the model.** MG-01 generalizes: any code path that mutates a model-produced field (rationale, reasoning, plan, decision narrative, tool-call arguments, final answer) is out of bounds. The controller may *reject* but must never *rewrite*.
- **Stream first, trim second.** MG-02: never trim before the operator sees output; only the planner-bound copy may be capped, and only at the outer assembly layer with a generous budget.
- **Keep the load-bearing differentiators.** The planner / synthesizer / retriever lane split, the manifold trace UI, the evidence-first turn loop, the AGENTS.md memory hierarchy, and the harness-profile capability negotiation all stay — only their names and the controller's overreach are in scope.
- **Local-first governance and execution policy stay in force.** Renames must not loosen the sandbox or approval surface.
- **One rename PR at a time.** Each Tier-1 rename in MG-03 lands as a separate, mechanically reviewable diff. Charter explicitly forbids one mega-PR.
- **Memory pinning.** The two operator preferences captured during this review (`feedback_trust_model_reasoning.md`, `feedback_stream_full_output.md`) are the canonical statements of MG-01 and MG-02 and must not be relaxed without explicit charter amendment.

## Halting Rules

- DO NOT halt while any MG-* goal has draft, planned, active, or verification-pending voyage/story work.
- HALT when MG-01 through MG-05 are complete and linked evidence proves: (a) planner rationale flows verbatim into traces, (b) shell output streams without the 1.2k cap, (c) all Tier-1 renames have landed and `cargo check` / `cargo test` / `keel doctor` are green, (d) `plan` mode and the new slash commands are wired, (e) docs match shipped capability.
- YIELD to the human if work would require: changing the planner / synthesizer / retriever architectural split; introducing a new bespoke abstraction; reintroducing controller-side rewrites of model output; reducing the streamed-output guarantee.

## Out Of Scope (explicit)

- Adding MCP server support (separate mission; descriptor exists today but is unwired).
- Implementing real subagents with their own context windows (separate mission; this mission only renames `specialist_brains` → `subagents` to free the namespace).
- Splitting `src/application/mod.rs` (17,556 lines) into separately-tested services. The `*Chamber` rename in MG-03 makes this easier but does not perform the split.
- Rewriting the manifold web UI; only the prose around it changes.
- Replacing `transit` or `sift`.
