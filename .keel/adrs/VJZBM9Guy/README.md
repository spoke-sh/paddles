---
# system-managed
id: VJZBM9Guy
index: 4
status: accepted
decided_at: 2026-05-13T21:41:45
supersedes: []
superseded-by: null
# authored
title: HTTP Model Clients As The Inference Boundary
context: "runtime"
applies-to: ["inference", "turn-runtime"]
mission: VJZ8ABnX2
---

# HTTP Model Clients As The Inference Boundary

## Status

**Accepted** — This ADR is the governing decision for model inference runtime
boundaries during mission `VJZ8ABnX2`.

## Context

Paddles historically supported in-process local model inference through the
Sift model provider. That made the application responsible for model loading,
model-path preparation, residency, tokenizer/model compatibility, hardware
placement, and runtime lifecycle concerns that are materially different from
the turn loop.

The current architecture is already centered on a recursive turn loop. The loop
owns interpretation, capability disclosure, action selection, governed local
execution, evidence accumulation, refinement, and final response handoff. Model
inference is a transport boundary that should be supplied by model clients, not
by application-owned model loading.

Local-first operation remains a requirement, but local-first does not require
in-process inference. Local models can run in an external local HTTP service
such as Ollama while Paddles keeps one model-client contract for both local and
remote providers.

## Decision

Paddles will use **HTTP model clients as the only supported inference boundary**
for action selection and final rendering.

This decision has four parts:

1. Paddles must not load inference models in-process for action selection or
   final rendering.
2. Local model inference is supported by local HTTP model services, with
   `ollama:<model>` as the canonical local provider form in docs, migration
   hints, and examples.
3. Legacy Sift model-provider configuration must fail before runtime
   construction with an actionable migration hint. It must not silently remap to
   Ollama or another provider because that would hide model, latency,
   deployment, and quality changes.
4. Sift retrieval/indexing is a separate concern. This ADR does not remove
   `sift-direct` or other retrieval/indexing behavior.

## Constraints

- **MUST:** Build action-selection and final-rendering inference through HTTP
  model-client adapters.
- **MUST:** Treat local model execution as an HTTP-hosted service boundary, not
  an in-process model loading boundary.
- **MUST:** Use `ollama:<model>` as the canonical local HTTP inference form when
  explaining migration from legacy local inference.
- **MUST:** Fail legacy `sift` model-provider config before runtime
  construction with a clear migration hint.
- **MUST NOT:** Silently remap `sift` model-provider selections to Ollama,
  OpenAI-compatible providers, or any other model provider.
- **MUST NOT:** Delete Sift retrieval/indexing as part of this inference ADR.
- **SHOULD:** Keep provider capability negotiation as the single place where
  transport and structured-output support are validated.

## Consequences

### Positive

- Model loading, hardware placement, batching, and model residency move out of
  the application runtime.
- Local and remote inference share one model-client architecture.
- Compatibility failures become explicit and debuggable instead of relying on
  hidden provider remaps.
- The turn loop remains the conceptual center while inference providers become
  replaceable transport clients.

### Negative

- Existing Sift model-provider users must start a local HTTP model service and
  update configuration.
- Runtime construction, config, tests, docs, and operator copy need coordinated
  migration.
- Some old local-model dependencies may stay until Sift inference code is
  removed in later sealed slices.

### Neutral

- Sift-backed retrieval/indexing can remain available while inference moves to
  HTTP clients.
- Internal terms such as planner, synthesizer, and gatherer are addressed by the
  related turn-runtime terminology cleanup, not by this ADR alone.

## Verification

| Check | Type | Description |
|-------|------|-------------|
| Legacy Sift provider fails early | automated | Config/runtime tests prove `sift` model-provider selections fail before runtime construction with an `ollama:<model>` migration hint |
| Inference runtime uses HTTP clients | automated | Runtime construction tests prove action-selection and final-rendering clients do not receive local `ModelPaths` |
| Sift retrieval remains separate | automated | Retrieval tests prove `sift-direct` can still prepare without Sift model-provider inference |
| Docs point to HTTP local inference | manual | Architecture and configuration docs cite this ADR and use `ollama:<model>` for local HTTP model examples |

## References

- Mission `VJZ8ABnX2` — HTTP-Only Inference And Turn Runtime Migration
- Research mission `VJYzz3ivM` — Turn Loop And HTTP Inference Cleanup
- `.keel/epics/VJZ0tpZQJ/voyages/VJZ14yp0U/CLEANUP_MIGRATION_RECOMMENDATION.md`
- `ARCHITECTURE.md`
- `CONFIGURATION.md`
