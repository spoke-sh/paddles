# Map Turn Loop And HTTP Inference Cleanup - Product Requirements

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
| GOAL-01 | Validate bearing recommendation in delivery flow | Adoption signal | Initial rollout complete |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Product/Delivery Owner | Coordinates planning and execution | Reliable strategic direction |

## Scope

### In Scope

- [SCOPE-01] Deliver the bearing-backed capability slice for this epic.

### Out of Scope

- [SCOPE-02] Unrelated platform-wide refactors outside bearing findings.

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Implement the core user workflow identified in bearing research. | GOAL-01 | must | Converts research recommendation into executable product capability. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Ensure deterministic behavior and operational visibility for the delivered workflow. | GOAL-01 | must | Keeps delivery safe and auditable during rollout. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

- Prove functional behavior through story-level verification evidence mapped to voyage requirements.
- Validate non-functional posture with operational checks and documented artifacts.

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Bearing findings reflect current user needs | Scope may need re-planning | Re-check feedback during first voyage |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Which Sift references are still load-bearing runtime behavior versus obsolete | Planner | Open |
| Does any current HTTP provider path still depend on planner/synthesizer lane | Planner | Open |
| What is the smallest first implementation slice that proves HTTP-only | Planner | Open |
| Which canonical docs own the post-cleanup architecture: ADR, ARCHITECTURE, | Planner | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] Inventory all Sift, embedded-model, and local model-loading references
- [ ] Inventory planner, synthesizer, and gatherer lane call sites and identify
- [ ] Produce a migration map with sealed implementation slices, test anchors,
- [ ] Surface any ADR-level decisions required before deleting shipped runtime
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
