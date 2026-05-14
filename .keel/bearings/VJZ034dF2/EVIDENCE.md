---
id: VJZ034dF2
---

# Map Turn Loop And HTTP Inference Cleanup — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:manual | conversation:2026-05-13 | 2026-05-13 | 2026-05-13 | high | high | Human requested a major cleanup focused on removing in-process Sift model loading in favor of HTTP inference and collapsing planner, synthesizer, and gatherer lanes around the turn loop before implementation. |
| SRC-02 | manual | manual:manual | src/infrastructure/adapters/sift_registry.rs | 2026-05-13 | 2026-05-13 | high | high | SiftRegistryAdapter owns supported local model IDs and calls sift::prepare_model inside spawn_blocking to return ModelPaths, so in-process model preparation is a concrete deletion/migration surface. |
| SRC-03 | manual | manual:manual | src/infrastructure/adapters/sift_agent.rs | 2026-05-13 | 2026-05-13 | high | high | SiftAgentAdapter imports candle and tokenizers, prepares Qwen model structs, and implements both local synthesis and planner action parsing, coupling inference lifecycle with runtime behavior. |
| SRC-04 | manual | manual:manual | src/application/mod.rs | 2026-05-13 | 2026-05-13 | high | high | RuntimeLaneConfig, PreparedModelLane, PreparedGathererLane, and prepare_runtime_lanes explicitly model planner, synthesizer, and gatherer lanes, and fetch ModelPaths for Sift-backed planner/synthesizer lanes. |
| SRC-05 | manual | manual:manual | src/main.rs | 2026-05-13 | 2026-05-13 | high | high | Main runtime factories branch ModelProvider::Sift to SiftAgentAdapter/SiftPlannerAdapter and non-Sift providers to HttpProviderAdapter/HttpPlannerAdapter, making the HTTP-only model-client boundary an obvious first migration seam. |
| SRC-06 | manual | manual:manual | src/application/agent_loop.rs | 2026-05-13 | 2026-05-13 | high | high | execute_recursive_planner_loop already drives action selection, governance, evidence accumulation, refinement, replan, and terminal outcomes from one turn-loop phase, so lane collapse should preserve this center and simplify surrounding names/config. |
| SRC-07 | manual | manual:manual | CONFIGURATION.md | 2026-05-13 | 2026-05-13 | high | high | Configuration docs still teach planner, synthesizer, and gatherer lane selection plus local Sift model defaults, so implementation cleanup must include owning documentation changes. |
## Feasibility

Feasible, but not as a single deletion pass. The code already has an HTTP
provider path for planner and synthesizer execution, and `src/main.rs` exposes
a narrow factory seam where Sift-backed engines can be replaced by HTTP-backed
model clients [SRC-05]. The hard part is not finding the seam; it is separating
three concerns that are currently interleaved: in-process local model
preparation, planner/synthesizer runtime lane configuration, and Sift-backed
retrieval/indexing [SRC-02] [SRC-03] [SRC-04].

The turn loop is already the strongest architectural center. The recursive loop
owns action selection, governance, evidence accumulation, refinement, replanning,
and terminal outcomes in one place, so the cleanup should collapse public lane
concepts into turn-loop phases without flattening useful internal boundaries
[SRC-06].

## Key Findings

1. The cleanup is human-directed and should produce recommendations before
   implementation, with HTTP-only inference and turn-loop-centered orchestration
   as the target direction [SRC-01].
2. In-process model loading is concrete, not just legacy prose: the Sift registry
   prepares local model artifacts through `sift::prepare_model`, and the Sift
   agent imports Candle/tokenizer model stacks directly [SRC-02] [SRC-03].
3. Planner, synthesizer, and gatherer are currently explicit runtime-lane
   concepts in configuration and prepared state, not only adapter labels
   [SRC-04].
4. Planner and synthesizer execution already share a branch point where Sift
   adapters can be removed in favor of HTTP-backed adapters, while retaining
   provider-specific wire-format negotiation [SRC-05].
5. Sift-backed retrieval is separate from Sift-backed model inference. The
   migration should decide whether `sift-direct` remains as a retrieval backend
   while `sift` disappears as a model provider [SRC-04] [SRC-05].
6. Foundational docs still describe local Sift models and planner/synthesizer/
   gatherer lane selection, so code cleanup without documentation cleanup would
   preserve operator drift [SRC-07].

## Unknowns

- Whether `sift-direct` should be retained as the default local retrieval/index
  backend or renamed to a provider-neutral retriever boundary in the same
  mission [SRC-04].
- Whether `--planner-provider`, `--planner-model`, and `--gatherer-provider`
  should be removed immediately or first mapped to compatibility aliases with
  explicit deprecation warnings [SRC-04] [SRC-07].
- Whether an ADR is required before deleting in-process model loading, because
  older verified missions explicitly shipped local model execution as a product
  capability [SRC-02] [SRC-03].
- How much of the `SynthesizerEngine` / `RecursivePlanner` port naming should
  survive internally versus being renamed to a unified model-client / turn-phase
  vocabulary [SRC-04] [SRC-06].
