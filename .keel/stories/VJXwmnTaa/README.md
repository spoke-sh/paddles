---
# system-managed
id: VJXwmnTaa
status: done
created_at: 2026-05-13T16:37:36
updated_at: 2026-05-13T17:16:43
# authored
title: Migrate Adapter Prompts To Agent Loop Vocabulary
type: feat
operator-signal:
scope: VJXwbmekZ/VJXwlG70U
index: 1
started_at: 2026-05-13T17:09:02
completed_at: 2026-05-13T17:16:43
---

# Migrate Adapter Prompts To Agent Loop Vocabulary

## Summary

Update Sift/local and HTTP/remote planner-facing prompts so they describe one
recursive agent loop and one bounded action-selection task, including the first
action.

## Acceptance Criteria

- [x] Sift and HTTP prompts describe the model as selecting bounded recursive agent actions inside the loop, not as choosing whether to enter a separate planner phase. [SRS-01/AC-01] <!-- verify: cargo test agent_loop_prompt_vocabulary --lib, SRS-01:start:end, proof: ac-1.log-->
- [x] Prompt tests fail if either lane reintroduces adapter-local first-action or recursive-action schema drift. [SRS-02/AC-02] <!-- verify: cargo test agent_loop_prompt_schema_parity --lib, SRS-02:start:end, proof: ac-2.log-->
- [x] Prompt tests keep terminal `answer`/`stop`, workspace actions, semantic actions, and `external_capability` in the unified action vocabulary gated by the capability manifest. [SRS-02/AC-03] <!-- verify: cargo test agent_loop_prompt_capability_manifest_split --lib, SRS-02:start:end, proof: ac-3.log-->
