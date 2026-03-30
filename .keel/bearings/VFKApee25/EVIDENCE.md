---
id: VFKApee25
---

# HTTP API Design For Paddles — Evidence

## Sources

| ID | Class | Provenance | Location | Observed / Published | Retrieved | Authority | Freshness | Notes |
|----|-------|------------|----------|----------------------|-----------|-----------|-----------|-------|
| SRC-01 | manual | manual:code-review | src/application/mod.rs | 2026-03-29 | 2026-03-29 | high | high | MechSuitService exposes session lifecycle, turn processing, and event sink injection |
| SRC-02 | manual | manual:code-review | src/domain/model/turns.rs | 2026-03-29 | 2026-03-29 | high | high | TurnEvent enum defines 16 typed event variants suitable for SSE streaming |
| SRC-03 | manual | manual:code-review | src/domain/model/traces.rs | 2026-03-29 | 2026-03-29 | high | high | TraceRecord with lineage DAG provides visualization graph structure |
| SRC-04 | manual | manual:code-review | Cargo.toml | 2026-03-29 | 2026-03-29 | high | high | tower 0.5 and tokio 1.43 already present as dependencies |

## Feasibility

Strong feasibility. The existing domain ports and application service already expose every primitive the HTTP adapter needs. TurnEventSink is the broadcast seam for SSE. TraceRecorder::replay provides history. No new domain types are required. The tokio async runtime and tower middleware are already in the dependency tree, making axum a natural fit.

## Key Findings

1. MechSuitService methods map directly to REST endpoints without new application logic [SRC-01]
2. TurnEvent variants serialize naturally to typed SSE payloads for real-time streaming [SRC-02]
3. TraceRecord lineage DAG (parent_record_id, branch_id) provides the graph structure needed for railroad visualization [SRC-03]
4. tower and tokio are already dependencies, making axum zero-friction to adopt [SRC-04]

## Unknowns

- Concurrent session handling when local model inference is single-threaded
- Whether trace replay should support cursor-based pagination for large histories
