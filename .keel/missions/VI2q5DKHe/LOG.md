# Idiomatic Agent Vocabulary And Trusted Model Reasoning - Decision Log

<!-- Append entries below. Each entry is an H2 with ISO timestamp. -->
<!-- Use `keel mission digest` to compress older entries when this file grows large. -->

## 2026-04-27T18:28:53

Charter authored from senior-staff review on 2026-04-27. Source: review session captured in memory entries feedback_trust_model_reasoning.md and feedback_stream_full_output.md. Five goals span (1) preserving model rationale verbatim, (2) streaming uncut tool output, (3) Tier-1 vocabulary renames toward industry-standard agent terms, (4) plan mode + slash command surface parity with Claude Code/Codex, (5) doc pruning to match shipped capability. Out of scope: MCP, real subagents, splitting application/mod.rs, transit/sift replacement.

## 2026-04-27T22:45:08

Mission achieved by local system user 'alex'

## 2026-04-28T20:12:32

Final verification review blocked; did not run `keel mission verify`. MG-03 is incomplete: targeted source/doc scan still finds `WorkspaceAction`, `PlannerAction`, `ExecutionHand`, `harness_profile`, `specialist_brains`, `gatherer`, `forensics`, `compaction_cue`, `premise_challenge`, `deliberation_signals`, and related retired vocabulary across src/tests/apps/docs. MG-05 is incomplete: foundational docs still contain mech suit/chamber/gatherer/forensic language and ARCHITECTURE.md still documents retired names as pending plus aspirational capabilities as roadmap items. Required follow-up: reopen/decompose the remaining rename and documentation truthfulness work before mission verification.
