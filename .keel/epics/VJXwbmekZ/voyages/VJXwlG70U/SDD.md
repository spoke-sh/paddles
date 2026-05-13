# Agent Vocabulary Cleanup - Software Design Description

> Update prompts, tests, and foundational docs so planning is described as model reasoning through bounded recursive agent actions, not as a separate architecture phase.

**SRS:** [SRS.md](SRS.md)

## Overview

After the contract and runtime migration, align operator-facing and
model-facing language with the architecture: Paddles has one recursive agent
loop. The model reasons by selecting bounded actions. Planning is not a separate
phase that decides whether the loop should start.

## Context & Boundaries

This voyage owns language, prompt, and documentation consistency. It should not
make runtime behavior changes except where tests require renaming prompt helper
surfaces.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Unified action contract | Domain/application behavior | Names and action vocabulary to document | VJXwlCA0P |
| Recursive loop migration | Application behavior | Runtime flow to document | VJXwlE718 |
| Foundational docs | Documentation | Owner of architecture and policy language | current repo |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Vocabulary | Prefer "recursive agent loop" and "bounded agent action" | Matches the product model. |
| Planning term | Use planning as the model's reasoning activity, not a separate phase name | Avoids the old architecture smell. |
| Verification | Use prompt tests and executable doc greps | Keeps language drift visible. |

## Architecture

Sift/local and HTTP/remote lanes may still be implementation lanes, but they
should receive the same action schema and describe the model's job consistently:
select the next bounded action in the recursive agent loop.

## Components

| Component | Purpose |
|-----------|---------|
| Adapter prompts | Model-facing wording for action selection |
| Prompt parity tests | Prevent lane drift and stale initial-routing language |
| Foundational docs | Operator-facing architecture, policy, and configuration contracts |

## Interfaces

No external runtime interfaces change in this voyage.

## Data Flow

1. Prompt builders embed the unified action schema.
2. Prompt tests assert lane parity and vocabulary constraints.
3. Foundational docs describe the same loop/action model.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Prompt still says first action is pre-loop routing | Prompt vocabulary test failure | Fail story | Revise prompt text |
| Docs imply planner phase is outside the loop | `rg` proof failure | Fail story | Revise owning doc |
