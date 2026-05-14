# Inventory Legacy Inference And Lane Surfaces - Software Design Description

> Produce the reviewed source inventory and migration recommendation for HTTP-only model inference and turn-loop-centered phase cleanup before implementation begins.

**SRS:** [SRS.md](SRS.md)

## Overview

This voyage is a behavior-neutral research slice. It produces the migration
map needed before removing in-process Sift model inference or collapsing
planner, synthesizer, and gatherer lane concepts. The work is split into three
inventories and one recommendation:

- Sift model-provider and model-loading surfaces
- HTTP provider/model-client seams
- lane and turn-loop phase surfaces
- sealed implementation slices with tests, compatibility handling, ADR/doc
  ownership, and human-review gate

## Context & Boundaries

The current runtime has both HTTP-backed provider adapters and Sift-backed
local model execution. It also exposes planner, synthesizer, and gatherer as
runtime lane concepts while the turn loop already owns most orchestration.
This voyage maps those facts; it does not implement the cleanup.

```
┌─────────────────────────────────────────┐
│        Research / Migration Map         │
│                                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐ │
│  │ Sift    │  │ HTTP    │  │ Turn    │ │
│  │ Model   │  │ Client  │  │ Loop    │ │
│  │ Map     │  │ Map     │  │ Map     │ │
│  └─────────┘  └─────────┘  └─────────┘ │
│                  ↓                      │
│          Migration Recommendation       │
└─────────────────────────────────────────┘
        ↑               ↑
   Source tree      .keel evidence
```

## Dependencies

| Dependency | Type | Purpose | Version/API |
|------------|------|---------|-------------|
| `rg` source scans | local tool | Locate Sift, model-provider, lane, turn-loop, config, prompt, and test references | repo-local |
| `keel` board artifacts | local CLI | Store mission, bearing, epic, voyage, stories, evidence, and lifecycle state | repo-local |
| Existing source/docs | repository | Evidence for inventory and migration recommendation | current worktree |

## Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| First slice shape | Research before implementation | The user requested recommendations before action, and this change crosses architecture, config, tests, and docs. |
| Model inference target | HTTP-only model-client boundary | Model loading and hardware lifecycle belong outside paddles; local models can still run behind an HTTP service. |
| Sift treatment | Separate Sift model inference from Sift retrieval/indexing | `sift-direct` may remain useful as a retrieval backend even if `sift` stops being a model provider. |
| Lane collapse | Collapse public lane concepts, preserve tested internal phases | The turn loop is already the runtime center; over-flattening internals would make behavior harder to test. |

## Architecture

The research output should classify each reference into one of these buckets:

- **Delete/migrate:** in-process model preparation, local Candle/Qwen execution,
  Sift-as-model-provider config, and Sift planner/synthesizer adapters.
- **Keep/rename later:** retrieval/indexing surfaces such as `sift-direct` when
  they are not responsible for model inference.
- **Collapse public concept:** planner/synthesizer/gatherer lane language in CLI,
  config, docs, and operator-facing state.
- **Preserve internal contract:** turn-loop phases, model-client request shaping,
  tool execution, evidence accumulation, final rendering, and tests that protect
  behavior.

## Components

| Component | Purpose | Output |
|-----------|---------|--------|
| Sift model inventory | Find local model-loading and Sift-as-model-provider surfaces | File list, dependency notes, deletion/migration classification |
| HTTP seam inventory | Find adapters and capability surfaces that can own inference transport | File list, reuse plan, compatibility notes |
| Lane/turn-loop inventory | Find lane concepts and map them to target turn-loop terminology | Public retirement map and internal preservation map |
| Migration recommendation | Turn evidence into implementation sequencing | Sealed slices, test anchors, docs/ADR list, review gate |

## Interfaces

No runtime API changes are introduced by this voyage. The research will refer to
existing Rust ports, CLI/config flags, and docs as evidence only.

## Data Flow

1. Run targeted source/doc searches.
2. Classify references by concern: inference transport, model loading,
   retrieval, turn-loop orchestration, public lane vocabulary, tests, docs.
3. Record proof in story evidence and the voyage report.
4. Present the recommended implementation slices for human review.

## Error Handling

| Error Condition | Detection | Response | Recovery |
|-----------------|-----------|----------|----------|
| Search finds mixed Sift model and retrieval references | Inventory classification conflict | Split the recommendation into separate model-provider and retrieval slices | Add an ADR/doc decision before implementation |
| Lane concept is load-bearing internally | Test/source review shows behavior depends on the split | Preserve the internal phase/helper and retire only public naming/config | Add focused tests to future implementation slice |
| Docs disagree with source reality | Source/doc inventory mismatch | Treat docs as cleanup targets, not authoritative behavior | Update owning docs in later implementation slices |
