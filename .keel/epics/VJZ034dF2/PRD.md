# HTTP-Only Inference And Turn Runtime Migration - Product Requirements

> Paddles can become more coherent by making HTTP inference the only model
transport boundary and by retiring planner, synthesizer, and gatherer as
separate runtime lane concepts. Local model execution remains possible when it
is hosted behind an HTTP service, but model loading, residency, batching, and
hardware placement should no longer be paddles-owned concerns.

## Problem Statement

Legacy architecture decisions are now pulling the codebase in competing
directions: older Sift/model-loading work treats in-process inference as an
application concern, while newer recursive-harness work frames the system
around a turn loop with live capability discovery and transport-specific model
clients. The research needs to map the current surfaces before implementation
so the cleanup can remove old concepts without breaking local-first behavior,
provider compatibility, traceability, or the model-owned reasoning contract.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Make model inference HTTP-only | Action-selection and final-rendering model calls use HTTP clients only | No in-process Sift inference path remains |
| GOAL-02 | Preserve local-first inference through Ollama-style HTTP services | Local setup examples and migration hints use `ollama:<model>` | Local-first docs no longer teach paddles-owned model loading |
| GOAL-03 | Hard-fail legacy Sift model-provider config | Old `sift` model-provider selections fail before runtime construction with a migration hint | No silent remapping to another provider |
| GOAL-04 | Collapse lane terminology across public and internal code | Planner/synthesizer/gatherer lane concepts are replaced by turn runtime phase and model-client names | Internal Rust types and user-facing surfaces no longer use lane-shaped vocabulary |
| GOAL-05 | Keep Sift retrieval out of scope | Sift retrieval/indexing remains independently selectable until a later decision | Retrieval tests remain green |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- [SCOPE-01] Add the ADR and compatibility policy for HTTP-only model inference.
- [SCOPE-02] Route action-selection and final-rendering model inference through HTTP model clients only.
- [SCOPE-03] Hard-fail legacy `sift` model-provider config with an actionable `ollama:<model>` migration hint.
- [SCOPE-04] Introduce turn-runtime model-client preferences and migrate away from lane-shaped provider state.
- [SCOPE-05] Delete in-process Sift model-loading and Sift inference adapters after HTTP-only construction is proven.
- [SCOPE-06] Rename user-facing and internal Rust planner/synthesizer/gatherer lane concepts to turn runtime phase concepts.
- [SCOPE-07] Update owning docs, prompts, tests, CLI/config help, and route/TUI copy in the same slices that change behavior.

### Out of Scope

- [SCOPE-08] Removing Sift retrieval/indexing.
- [SCOPE-09] Replacing the recursive turn loop or model-owned reasoning contract with controller-authored pseudo-plans.
- [SCOPE-10] Silent compatibility remaps from Sift inference config to Ollama or any other HTTP provider.
- [SCOPE-11] Unrelated feature work or broad platform refactors outside the migration.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Record an ADR that makes HTTP model clients the only supported inference boundary for action selection and final rendering. | GOAL-01, GOAL-02 | must | Model loading, hardware placement, batching, and residency should not be paddles-owned concerns. |
| FR-02 | Reject legacy `sift` model-provider config before runtime construction with a migration hint that names `ollama:<model>`. | GOAL-03 | must | Silent remapping would hide deployment and model-quality changes. |
| FR-03 | Build action-selection and final-rendering model clients exclusively from HTTP provider configuration. | GOAL-01 | must | The runtime should have one model-client boundary. |
| FR-04 | Preserve Sift retrieval/indexing independently from model inference removal. | GOAL-05 | must | Retrieval has a different blast radius and needs a later decision. |
| FR-05 | Replace runtime lane preferences with turn-runtime model-client preferences while keeping legacy lane-shaped config readable only for migration. | GOAL-02, GOAL-04 | must | New configuration should teach the target architecture. |
| FR-06 | Delete in-process Sift inference adapters, model registry preparation, and inference-only dependencies once HTTP-only construction is proven. | GOAL-01, GOAL-03 | must | Dead inference code should not remain as a maintenance branch. |
| FR-07 | Rename public and internal planner/synthesizer/gatherer lane concepts to turn runtime phases, model clients, retrieval, execution, evidence, and final rendering. | GOAL-04 | must | The codebase should be logically centered on the turn loop. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Every implementation story follows TDD with a failing test or doc check before behavior changes. | GOAL-01, GOAL-03, GOAL-04 | must | This migration crosses runtime construction, config, docs, and user-facing language. |
| NFR-02 | Provider compatibility failures must be deterministic, actionable, and occur before model runtime construction. | GOAL-03 | must | Operators need clear migration errors, not late runtime surprises. |
| NFR-03 | Docs and source vocabulary must remain synchronized in each sealed slice. | GOAL-02, GOAL-04 | must | Documentation is part of the migration surface. |
| NFR-04 | Existing HTTP provider behavior, retries, structured final answers, credential boundaries, and local-first operation remain covered by tests. | GOAL-01, GOAL-02 | must | The cleanup should simplify architecture without losing provider coverage. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| The research recommendation remains the canonical implementation plan | Scope may need re-planning | Human confirmed slices 1-5 on 2026-05-13 |
| Sift retrieval/indexing can remain independent of Sift inference deletion | Retrieval follow-up mission may need to move earlier | Keep retrieval tests green during inference slices |
| Internal Rust renames can be completed without replacing the turn loop itself | Scope may need additional refactor stories | Use compiler and test failures to expose remaining references |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Does any Sift retrieval code still depend on inference-only dependencies after the model adapters are removed? | Operator | Open |
| Which compatibility aliases should be removed immediately versus retained as warning-only shims during the preference migration? | Operator | Open |
| How many internal port renames can land safely in one story without obscuring behavior changes? | Operator | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] ADR accepted and owning docs identify HTTP model clients as the only inference boundary.
- [ ] Legacy `sift` model-provider config hard-fails with a migration hint naming `ollama:<model>`.
- [ ] Action-selection and final-rendering runtime construction no longer resolves local `ModelPaths`.
- [ ] In-process Sift inference adapters, model preparation, and inference-only dependencies are removed.
- [ ] New runtime preferences use turn-runtime/model-client terminology; legacy lane-shaped config is migration input only.
- [ ] User-facing copy and internal Rust code no longer present planner/synthesizer/gatherer as runtime lanes.
- [ ] Sift retrieval/indexing remains independently selectable and tested.
<!-- END SUCCESS_CRITERIA -->

## Research Analysis

*From bearing assessment:*

## Findings


- Proceeding is justified, but the first active slice should remain research and
  architecture mapping rather than deleting Sift code immediately [SRC-01]
  [SRC-02] [SRC-03].
- HTTP-only inference has an existing migration seam in `src/main.rs`, where
  Sift planner/synthesizer factories can be replaced by HTTP-backed factories
  while preserving provider wire-format negotiation [SRC-05].
- The turn loop already owns the important runtime behavior, so lane collapse
  should reduce public configuration and naming while keeping tested phase
  boundaries around action selection, retrieval, execution, and synthesis
  [SRC-04] [SRC-06].
- Sift retrieval and Sift model inference must be separated explicitly. The
  cleanup can remove Sift as a model provider without necessarily removing the
  `sift-direct` retrieval/index backend in the same slice [SRC-04] [SRC-05].
- Documentation is part of the cleanup surface because CONFIGURATION currently
  teaches local Sift models and lane-specific operation [SRC-07].


## Opportunity Cost


This will delay feature work while the runtime vocabulary and inference boundary
are stabilized. That cost is acceptable because the current code and docs encode
legacy assumptions that will keep multiplying migration work if left in place
[SRC-04] [SRC-07].


## Dependencies


- A source inventory that distinguishes Sift-as-model-provider from Sift-as-
  retrieval-backend before deletion work begins [SRC-02] [SRC-03] [SRC-04].
- A migration map that identifies the first testable implementation slice,
  preferably the HTTP-only model-provider boundary before broader lane collapse
  [SRC-05].
- A decision on whether to record an ADR for deleting in-process local model
  hosting from paddles [SRC-01] [SRC-02] [SRC-03].
- Foundational documentation updates in the same sealed slices that change
  runtime behavior or operator configuration [SRC-07].


## Alternatives Considered


- Keep Sift local model inference and only rename lanes: rejected because it
  leaves model loading, hardware residency, and inference lifecycle inside
  paddles, which is the core concern raised by the cleanup [SRC-01] [SRC-02]
  [SRC-03].
- Delete every Sift reference immediately: rejected because `sift-direct`
  retrieval may still be a legitimate local index backend even if Sift is
  removed as a model provider [SRC-04] [SRC-05].
- Collapse planner/synthesizer/gatherer by merging all code into one runtime
  object: rejected because the turn loop already centralizes orchestration, and
  the better cleanup is public-concept simplification with internal phase
  boundaries preserved [SRC-06].

## Research Provenance

*Source records from bearing evidence:*

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:manual | conversation:2026-05-13 | 2026-05-13 | 2026-05-13 | high | high | Human requested a major cleanup focused on removing in-process Sift model loading in favor of HTTP inference and collapsing planner, synthesizer, and gatherer lanes around the turn loop before implementation. |
| SRC-02 | manual | manual:manual | src/infrastructure/adapters/sift_registry.rs | 2026-05-13 | 2026-05-13 | high | high | SiftRegistryAdapter owns supported local model IDs and calls sift::prepare_model inside spawn_blocking to return ModelPaths, so in-process model preparation is a concrete deletion/migration surface. |
| SRC-03 | manual | manual:manual | src/infrastructure/adapters/sift_agent.rs | 2026-05-13 | 2026-05-13 | high | high | SiftAgentAdapter imports candle and tokenizers, prepares Qwen model structs, and implements both local synthesis and planner action parsing, coupling inference lifecycle with runtime behavior. |
| SRC-04 | manual | manual:manual | src/application/mod.rs | 2026-05-13 | 2026-05-13 | high | high | RuntimeLaneConfig, PreparedModelLane, PreparedGathererLane, and prepare_runtime_lanes explicitly model planner, synthesizer, and gatherer lanes, and fetch ModelPaths for Sift-backed planner/synthesizer lanes. |
| SRC-05 | manual | manual:manual | src/main.rs | 2026-05-13 | 2026-05-13 | high | high | Main runtime factories branch ModelProvider::Sift to SiftAgentAdapter/SiftPlannerAdapter and non-Sift providers to HttpProviderAdapter/HttpPlannerAdapter, making the HTTP-only model-client boundary an obvious first migration seam. |
| SRC-06 | manual | manual:manual | src/application/agent_loop.rs | 2026-05-13 | 2026-05-13 | high | high | execute_recursive_planner_loop already drives action selection, governance, evidence accumulation, refinement, replan, and terminal outcomes from one turn-loop phase, so lane collapse should preserve this center and simplify surrounding names/config. |
| SRC-07 | manual | manual:manual | CONFIGURATION.md | 2026-05-13 | 2026-05-13 | high | high | Configuration docs still teach planner, synthesizer, and gatherer lane selection plus local Sift model defaults, so implementation cleanup must include owning documentation changes. |

---

*This PRD was seeded from bearing `VJZ034dF2`. See `bearings/VJZ034dF2/` for original research.*
