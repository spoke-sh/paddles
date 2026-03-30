---
id: VFKApee25
---

# HTTP API Design For Paddles — Assessment

## Scoring Factors

| Factor | Score | Rationale |
|--------|-------|-----------|
| Impact | 5 | Unlocks web interface, visualization, and external client access to the harness |
| Confidence | 5 | Implemented and verified: axum server, SSE streaming, trace graph endpoint all working |
| Effort | 2 | Existing domain ports mapped directly to HTTP endpoints with no new application logic |
| Risk | 1 | Infrastructure adapter only, no domain changes, all 90 tests pass unchanged |

## Findings

- MechSuitService methods map directly to REST endpoints without new application logic [SRC-01]
- TurnEvent Serialize derive enables typed SSE payloads with zero manual serialization [SRC-02]
- TraceRecord lineage DAG provides the graph structure needed for railroad visualization [SRC-03]
- axum integrates with existing tokio runtime and tower middleware at zero friction [SRC-04]

## Opportunity Cost

Minimal. The HTTP server runs as a background tokio task alongside the CLI/TUI. No existing functionality was modified or displaced.

## Dependencies

- tokio async runtime (already present) [SRC-04]
- tower-http for CORS (new, lightweight) [SRC-04]
- axum 0.8 for HTTP framework (new) [SRC-04]

## Alternatives Considered

- WebSocket instead of SSE: rejected because TurnEvent flow is unidirectional (server to client), SSE is HTTP-native with browser auto-reconnection, and no upgrade negotiation is needed [SRC-02]
- Separate process instead of shared MechSuitService: rejected because local model inference is sequential regardless of interface, and sharing the service avoids duplicate model loading [SRC-01]

## Recommendation

[x] Proceed: converted to epic VFKBCVjpo, implemented and delivered [SRC-01]
[ ] Park
[ ] Decline
