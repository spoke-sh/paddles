# Create Recursive Harness Eval Suite - Software Design Description

> Create a local eval harness and initial corpus proving recursive evidence gathering, tool failure recovery, edit obligations, delegation, context pressure, and architecture boundaries.

**SRS:** [SRS.md](SRS.md)

## Overview

Add a deterministic eval harness that drives Paddles through fixed local scenarios and asserts evidence, event, and architecture outcomes. Evals complement unit tests by checking end-to-end harness contracts.

## Context & Boundaries

Domain owns scenario and assertion vocabulary. Application services execute eval cases through existing ports. Infrastructure supplies fixture workspaces, fake providers, and report formatting.

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| Existing tests | development | Reuse fake providers and harness fixtures | internal |
| Trace/evidence models | domain | Assertion targets | internal |
| Cargo test/CLI | tooling | Local execution | stable |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First runner | local deterministic runner | Avoids network and model variability. |
| First corpus | harness contracts over model quality | Protects architecture behavior. |
| Reporting | structured plus human-readable | Supports CI and operator use. |

## Components

| Component | Layer | Responsibility |
|-----------|-------|----------------|
| EvalScenario | domain | Inputs, constraints, and expected assertions. |
| EvalRunner | application | Execute scenarios through ports. |
| EvalFixtureAdapter | infrastructure | Local workspaces and fake providers. |

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Scenario fixture missing | load failure | Fail fast with fixture path | Restore fixture. |
| Assertion mismatch | eval assertion | Report violated contract | Inspect trace and fix regression. |
| External dependency requested | offline guard | Fail scenario | Mark dependency explicit or mock it. |
