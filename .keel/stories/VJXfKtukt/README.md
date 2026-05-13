---
# system-managed
id: VJXfKtukt
status: done
created_at: 2026-05-13T15:28:17
updated_at: 2026-05-13T15:52:05
# authored
title: Migrate Planner Prompts To Shared Schema
type: feat
operator-signal:
scope: VJXeteRQ5/VJXf4hlYW
index: 1
started_at: 2026-05-13T15:45:16
completed_at: 2026-05-13T15:52:05
---

# Migrate Planner Prompts To Shared Schema

## Summary

Migrate Sift/local and HTTP/remote planner prompt builders so action-schema
text comes from the shared renderer. Provider-specific transport instructions
remain adapter-local.

## Acceptance Criteria

- [x] Sift initial, recursive, retry, and redecision prompts consume the shared schema renderer. [SRS-01/AC-01] <!-- verify: cargo test sift_planner_prompts_use_shared_action_schema_renderer --lib, SRS-01:start:end, proof: ac-1.log-->
- [x] HTTP planner prompts consume the shared schema renderer while preserving native-tool, structured JSON, and prompt-envelope transport instructions. [SRS-02/AC-02] <!-- verify: cargo test planner_system_prompt_demands_complete_json_action_envelopes --lib, SRS-02:start:end, proof: ac-2.log-->
- [x] `rg` or equivalent proof shows no remaining adapter-local planner action JSON example lists outside the shared renderer. [SRS-03/AC-03] <!-- verify: zsh -lc 'cd /home/alex/workspace/spoke-sh/paddles && if rg -n "Allowed actions:|## Action Schema|Available actions:" src/infrastructure/adapters; then exit 1; else exit 0; fi', SRS-03:start:end, proof: ac-3.log-->
