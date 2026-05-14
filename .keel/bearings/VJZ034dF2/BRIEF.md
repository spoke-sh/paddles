# Map Turn Loop And HTTP Inference Cleanup — Brief

## Hypothesis

Paddles can become more coherent by making HTTP inference the only model
transport boundary and by retiring planner, synthesizer, and gatherer as
separate runtime lane concepts. Local model execution remains possible when it
is hosted behind an HTTP service, but model loading, residency, batching, and
hardware placement should no longer be paddles-owned concerns.

## Problem Space

Legacy architecture decisions are now pulling the codebase in competing
directions: older Sift/model-loading work treats in-process inference as an
application concern, while newer recursive-harness work frames the system
around a turn loop with live capability discovery and transport-specific model
clients. The research needs to map the current surfaces before implementation
so the cleanup can remove old concepts without breaking local-first behavior,
provider compatibility, traceability, or the model-owned reasoning contract.

## Success Criteria

- [ ] Inventory all Sift, embedded-model, and local model-loading references
      that would need migration or deletion.
- [ ] Inventory planner, synthesizer, and gatherer lane call sites and identify
      which concepts should collapse into turn-loop phases versus remain as
      internal helpers.
- [ ] Produce a migration map with sealed implementation slices, test anchors,
      and owning documentation updates before code changes begin.
- [ ] Surface any ADR-level decisions required before deleting shipped runtime
      paths.

## Open Questions

- Which Sift references are still load-bearing runtime behavior versus obsolete
  naming, docs, tests, or compatibility scaffolding?
- Does any current HTTP provider path still depend on planner/synthesizer lane
  naming or split prompts in a way that must be preserved temporarily?
- What is the smallest first implementation slice that proves HTTP-only
  inference boundaries without combining unrelated lane cleanup?
- Which canonical docs own the post-cleanup architecture: ADR, ARCHITECTURE,
  POLICY, CONFIGURATION, README, and/or active mission artifacts?
