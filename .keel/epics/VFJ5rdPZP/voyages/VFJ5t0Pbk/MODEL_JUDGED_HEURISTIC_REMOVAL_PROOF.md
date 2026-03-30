# Model-Judged Heuristic Removal Proof

## Summary

Mission `VFJ5rckZO` removes the remaining reasoning-heavy controller heuristics
from the primary `paddles` harness path and makes the model own interpretation,
re-decision, and retrieval choices wherever those choices are about judgement
rather than safety.

The delivered slice does five concrete things:

1. removes legacy direct-path string heuristics from the active adapter/runtime
   path
2. replaces lexical interpretation scoring with model-judged guidance selection
   from an `AGENTS.md`-rooted document graph
3. replaces heuristic initial/planner fallback ranking with one more
   constrained model re-decision pass, then fail-closed controller fallback
4. passes model-selected retrieval mode and retrieval strategy through the
   recursive planner/gatherer boundary
5. removes hardcoded source-priority evidence ranking in favor of evidence
   order/rank produced by the gatherer or planner

## Code Proof

- Primary model-judged interpretation and re-decision:
  - `src/infrastructure/adapters/sift_agent.rs`
  - `derive_interpretation_context`
  - `select_initial_action`
  - `select_planner_action`
- `AGENTS.md` as the only hardcoded interpretation root:
  - `src/infrastructure/adapters/agent_memory.rs`
- Retrieval mode/strategy carried as typed planner intent:
  - `src/domain/ports/context_gathering.rs`
  - `src/domain/ports/planning.rs`
  - `src/application/mod.rs`
- Gatherers honoring planner-selected retrieval strategy:
  - `src/infrastructure/adapters/sift_context_gatherer.rs`
  - `src/infrastructure/adapters/sift_autonomous_gatherer.rs`
- Foundational boundary updates:
  - `README.md`
  - `ARCHITECTURE.md`
  - `INSTRUCTIONS.md`

## Behavioral Proof

The regression suite now proves the intended boundary directly.

Interpretation and guidance selection:

- `infrastructure::adapters::sift_agent::tests::interpretation_context_expands_model_selected_guidance_subgraph_from_agents_roots`
  proves the planner expands an `AGENTS.md`-rooted guidance graph through
  model-selected edges before assembling interpretation context.
- `infrastructure::adapters::agent_memory::tests::agent_memory_roots_only_load_agents_documents`
  proves `AGENTS.md` remains the only hardcoded interpretation root.

Initial action and next-step re-decision:

- `infrastructure::adapters::sift_agent::tests::invalid_initial_action_replies_use_constrained_redecision_before_succeeding`
- `infrastructure::adapters::sift_agent::tests::invalid_initial_action_replies_fail_closed_after_redecision_is_still_invalid`
- `infrastructure::adapters::sift_agent::tests::invalid_planner_replies_use_constrained_redecision_before_succeeding`
- `infrastructure::adapters::sift_agent::tests::invalid_planner_replies_fail_closed_after_redecision_is_still_invalid`

These four tests prove the controller no longer ranks hint lists or infers the
next step heuristically after invalid planner JSON. It asks the model again in a
constrained schema, then stops cleanly if the reply is still invalid.

Direct-path tool/runtime behavior:

- `infrastructure::adapters::sift_agent::tests::deterministic_action_turns_require_model_selected_tool_calls`
- `infrastructure::adapters::sift_agent::tests::action_prompts_retry_for_tool_calls_after_prose`
- `infrastructure::adapters::sift_agent::tests::action_prompts_retry_for_tool_calls_after_empty_response`

These prove the controller no longer infers shell commands or tool calls from
plain user text on the direct adapter path. Deterministic action turns now
depend on explicit model-emitted tool JSON plus controller validation.

Retrieval/evidence behavior:

- `application::tests::recursive_search_requests_graph_mode_and_surfaces_graph_trace_summary`
  proves recursive `search` actions carry model-selected retrieval mode and
  strategy into the gatherer boundary.
- `infrastructure::adapters::sift_agent::tests::grounded_answer_fallback_preserves_evidence_order_without_source_priority`
  proves grounded fallback now uses evidence order/rank instead of a hardcoded
  source-priority controller ranking.

## Verification

Executed locally:

```bash
cargo test -q
just quality
cargo nextest run
```

Result:

- 90 tests passed under `cargo test -q`
- `cargo fmt --all --check` and `cargo clippy --all-targets --all-features -- -D warnings` passed through `just quality`
- 90 tests passed under `cargo nextest run`

## Controller-Versus-Model Boundary After This Mission

Model-owned judgement:

- guidance graph expansion from `AGENTS.md` roots
- interpretation document/tool-hint/procedure selection
- initial bounded action selection
- recursive next-step selection
- retrieval mode and retrieval strategy for recursive search/refine

Controller-owned constraints:

- schema parsing and validation
- safe command allowlists
- path validation
- loop/tool/read/inspect budgets
- deterministic tool execution
- fail-closed fallback when repeated model replies remain invalid

## Remaining Gaps

- Recursive `search` / `refine` still lower through the configured gatherer
  backend rather than a richer unified planner resource graph.
- Evidence ordering now follows gatherer/planner rank, but the local models can
  still produce weak grounded synthesis; that remains a model-quality problem,
  not a controller-heuristic routing problem.
