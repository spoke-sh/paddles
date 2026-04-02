# Transit-First Web Forensics Inspector - Product Requirements

## Problem Statement

The current web UI exposes transcript rows and a lossy transit trace board, but operators cannot inspect exact assembled context, provider request envelopes, raw model outputs, rendered outputs, force snapshots, or context lineage from transit as the source of truth. We need a context-lineage-first web forensic inspector backed by transit artifacts that also supports live active-turn debugging.

## Goals & Objectives

| ID | Goal | Success Metric | Target |
|----|------|----------------|--------|
| GOAL-01 | Exact transit artifact fidelity: all relevant model exchange artifacts can be replayed exactly from transit for forensic inspection. | Operators can inspect assembled context, redacted provider request envelopes, raw provider responses, and rendered outputs for a turn without relying on UI-local reconstruction | First voyage |
| GOAL-02 | Context-lineage-first navigation: the web UI centers artifact lineage rather than transcript order alone. | One inspector can navigate conversation, turn, model call, planner step, trace record, and artifact lineage in a coherent sequence | First voyage |
| GOAL-03 | Default-visible force inspection: the UI shows what forces were applied and which sources contributed to them. | Pressure, truncation/compaction, execution/edit pressure, fallback/coercion, and budget effects are visible by default with contribution estimates by source | First voyage |
| GOAL-04 | Live and comparative debugging: active turns and shadow comparisons are inspectable, not just completed turns. | Provisional artifacts stream during active turns and the inspector can compare current state to a shadow baseline from lineage | First voyage |

## Users

| Persona | Description | Primary Need |
|---------|-------------|--------------|
| Operator | Developer running paddles through the web UI during active or completed turns | Inspect exact context assembly and model exchange behavior without leaving the browser |
| Maintainer | Engineer debugging planner, synthesis, pressure, or compaction behavior | A coherent forensic surface rooted in stored transit artifacts rather than inferred UI state |

## Scope

### In Scope

- [SCOPE-01] Transit capture of exact assembled planner/synth/context artifacts, redaction-safe provider request envelopes, raw provider responses, and rendered outputs
- [SCOPE-02] Transit capture of context lineage edges, force snapshots, and contribution estimates by source
- [SCOPE-03] Conversation- and turn-scoped replay/live projection APIs for transit forensic artifacts, including provisional and final artifact states
- [SCOPE-04] A dense web forensic inspector with context-lineage-first navigation and raw/rendered toggles
- [SCOPE-05] A secondary interactive overview for force/topology/shadow comparison above the precise 2D inspector
- [SCOPE-06] Live active-turn streaming of provisional forensic artifacts in coherent sequence

### Out of Scope

- [SCOPE-07] TUI parity for the forensic inspector
- [SCOPE-08] Changes to planner, gatherer, or synthesis decision-making beyond recording and displaying their artifacts
- [SCOPE-09] External telemetry backends or remote multi-user synchronization
- [SCOPE-10] Purely decorative globe or ambient views that are not tied to inspectable transit data

## Requirements

### Functional Requirements

<!-- BEGIN FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| FR-01 | Transit must record exact model exchange artifacts for planner/synth/model calls, including assembled context, redacted provider request envelopes, raw provider responses, and normalized/rendered outputs. | GOAL-01 | must | The forensic UI cannot be authoritative if exact artifacts are not stored in transit first. |
| FR-02 | Transit must record context lineage relationships between conversation, turn, model call, planner step, artifacts, and resulting outputs. | GOAL-01, GOAL-02 | must | Context-lineage-first navigation depends on durable lineage edges instead of UI reconstruction. |
| FR-03 | Transit must record force snapshots and estimated contribution by source for applied pressure, truncation/compaction, execution/edit pressure, fallback/coercion, and budget effects. | GOAL-03 | must | Operators need to see not only that force was applied, but why and from which sources. |
| FR-04 | The application/web layer must expose replay and live update projections for forensic transit artifacts, including provisional, superseded, and final artifact states. | GOAL-01, GOAL-04 | must | The browser needs an exact source of truth for completed and active turns. |
| FR-05 | The web UI must provide a context-lineage-first dense inspector that can navigate conversation, turn, model call, planner loop step, trace record, and artifact sequence in one unified surface. | GOAL-02 | must | The primary navigation model is lineage, not transcript order alone. |
| FR-06 | The web UI must support toggling between exact raw content and format-friendly rendered views for inspectable artifacts. | GOAL-01, GOAL-02 | must | Operators need both byte-faithful debugging and readable inspection. |
| FR-07 | The web UI must show applied forces and contribution-by-source by default for the current selection. | GOAL-03 | must | Force visibility should not require hidden tabs or optional debug toggles. |
| FR-08 | The web UI must provide a secondary interactive overview for topology/force inspection and shadow comparison without replacing the precise 2D inspector. | GOAL-03, GOAL-04 | should | A compact overview helps operators reason about structure and change, but exact inspection still belongs in 2D. |
| FR-09 | Active turns must stream provisional forensic artifacts into the web inspector in coherent sequence and reconcile them as final records arrive. | GOAL-04 | should | The operator wants live forensics, not only post-turn replay. |
<!-- END FUNCTIONAL_REQUIREMENTS -->

### Non-Functional Requirements

<!-- BEGIN NON_FUNCTIONAL_REQUIREMENTS -->
| ID | Requirement | Goals | Priority | Rationale |
|----|-------------|-------|----------|-----------|
| NFR-01 | Transit artifacts must be sufficient to rebuild the forensic inspector after missed live updates, without DOM repair heuristics or replaying from non-transit UI state. | GOAL-01, GOAL-04 | must | Transit is intended to become the authoritative substrate for this feature. |
| NFR-02 | Auth headers and obvious secrets must be redacted before browser exposure while preserving exact payload bodies otherwise. | GOAL-01 | must | The user wants exact payload inspection, but not accidental secret leakage. |
| NFR-03 | Active-turn and completed-turn updates must appear in the web UI without reload. | GOAL-04 | must | Reload-based debugging defeats the purpose of live forensic inspection. |
| NFR-04 | Dense dev-mode inspection must remain usable for long conversations and large artifact payloads. | GOAL-02, GOAL-03 | should | The first audience is maintainers investigating complex turns, not lightweight end-user browsing. |
| NFR-05 | The implementation must remain local-first and avoid mandatory hosted services; any visualization library must be served locally or vendored. | GOAL-04 | must | Preserves the repo's local-first operating constraints. |
| NFR-06 | The forensic inspector remains web-only in this epic and does not require matching TUI changes. | GOAL-02, GOAL-04 | should | Keeps the initial slice focused and reduces cross-surface scope. |
<!-- END NON_FUNCTIONAL_REQUIREMENTS -->

## Verification Strategy

| Area | Method | Evidence |
|------|--------|----------|
| Transit artifact fidelity | Unit/integration tests around trace recording and replay ordering | Story-level test evidence |
| Force and lineage capture | Unit/integration tests around stored snapshots and contribution estimates | Story-level test evidence |
| Web replay/live projection | Endpoint/SSE projection tests plus manual active-turn inspection | Story-level test evidence and UI proof |
| Dense inspector UX | Manual browser verification against completed and active turns | Story-level manual evidence |

## Assumptions

| Assumption | Impact if Wrong | Validation |
|------------|-----------------|------------|
| Transit can be extended to store richer artifact records without breaking existing trace replay consumers. | We may need a parallel forensic journal instead of extending transit. | Validate during the first transit-capture story. |
| Provider request/response bodies are available at the right seam to capture exact envelopes before web projection. | Exact payload inspection may need adapter-specific fallback handling. | Validate during artifact capture design. |
| A dense inspector and a secondary overview can fit the existing web surface without requiring a wholesale frontend rewrite. | We may need to stage the UI across multiple iterations or introduce a local build pipeline. | Validate during web design and implementation. |

## Open Questions & Risks

| Question/Risk | Owner | Status |
|---------------|-------|--------|
| Should the forensic inspector replace the current trace board by default or live beside it first? | UX / web | Open |
| Which secondary overview library best fits the current local-first web stack: plain SVG/canvas, force-graph, Three.js, or another locally served dependency? | Web / architecture | Open |
| Should shadow comparison stop at previous-lineage baseline for v1, or include hypothetical slider-driven projections in the first voyage? | Product / web | Open |
| How expensive will contribution-estimate computation be if captured for every step of an active turn? | Application / performance | Open |

## Success Criteria

<!-- BEGIN SUCCESS_CRITERIA -->
- [ ] A completed turn can be inspected from the web UI with exact raw and rendered artifacts replayed directly from transit
- [ ] Default inspector panels show applied forces and contribution-by-source without extra debug toggles
- [ ] Active turns stream provisional forensic artifacts into the web UI without reload and reconcile them to final records
- [ ] The web inspector navigates context lineage directly across turn, model call, planner step, and artifact sequence
<!-- END SUCCESS_CRITERIA -->
