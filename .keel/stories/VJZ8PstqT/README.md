---
# system-managed
id: VJZ8PstqT
status: done
created_at: 2026-05-13T21:30:04
updated_at: 2026-05-13T23:06:01
# authored
title: Rename Planner Synthesizer Gatherer Ports To Turn Phases
type: refactor
operator-signal:
scope: VJZ034dF2/VJZ8ERr2f
index: 2
started_at: 2026-05-13T22:59:29
completed_at: 2026-05-13T23:06:01
---

# Rename Planner Synthesizer Gatherer Ports To Turn Phases

## Summary

Rename planner, synthesizer, and gatherer ports/modules where they encode the
old lane architecture. Preserve behavior under clearer turn phase names such as
action selection, final rendering, retrieval, and evidence.

## Acceptance Criteria

- [x] Internal planner/synthesizer/gatherer names are replaced where they describe lane architecture rather than unavoidable compatibility. [SRS-02/AC-01] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && ! rg -n "RecursivePlanner|SynthesizerEngine|SynthesisHandoff|ContextGatherer|GathererCapability|SynthesizerFactory|PlannerFactory|GathererFactory|HttpPlannerAdapter|build_synthesizer_engine|build_planner_engine|synthesizer_engine|planner_engine|context_gathering|sift_context_gatherer|sift_direct_gatherer|sift_autonomous_gatherer|context1_gatherer|SiftContextGathererAdapter|SiftDirectGathererAdapter|SiftAutonomousGathererAdapter|Context1GathererAdapter" src && ! rg -n "synthesizer lane|planner lane|gatherer lane|context-gathering lane|synthesizer engine|context-gathering subagents|no retrieval_provider|retrieval_provider requests|retrieval_provider backend|retrieval_provider is configured|Checked retrieval_provider" src/domain/ports src/application src/infrastructure/adapters src/infrastructure/runtime_presentation.rs src/main.rs --glob "!http_provider.rs"', SRS-02:start:end, proof: ac-1.log-->
- [x] The turn loop still exposes tested behavior for action selection, retrieval, execution, evidence accumulation, refinement, and final rendering. [SRS-02/AC-02] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --lib unified_loop && cargo test --lib execute_planner_gather_step && cargo nextest run action_selection_client_builds_from_http_provider_configuration final_rendering_client_builds_from_http_provider_configuration direct_gatherer_returns_direct_retrieval_metadata_and_evidence --no-tests pass', SRS-02:start:end, proof: ac-2.log-->
- [x] Prompt and execution-contract tests continue to expose live capabilities and enforced constraints without synthetic controller-authored plans. [SRS-02/AC-03] <!-- verify: sh -lc 'cd "$(git rev-parse --show-toplevel)" && cargo test --lib execution_contract && cargo test --lib runtime_posture_projection && cargo test agent_loop_prompt --all-targets', SRS-02:start:end, proof: ac-3.log-->
