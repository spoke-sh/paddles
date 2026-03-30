# HTTP API Design For Paddles — Brief

## Hypothesis

Paddles can serve its recursive planning harness over HTTP without introducing a new application layer. The existing domain ports (SynthesizerEngine, RecursivePlanner, TraceRecorder, TurnEventSink) are sufficient to power both a chat-style web interface and a real-time trace visualization, with SSE streaming TurnEvents to connected clients as they happen.

## Problem Space

The CLI and TUI are the only interfaces into the paddles harness today. A web interface broadens access and enables rich visualization that terminals cannot support, specifically a railroad-style DAG view of transit trace streams showing branch/merge lineage, planner actions, and checkpoint status in real time.

The API design must answer: what is the right HTTP surface for a recursive in-context planning harness where turns are long-lived, events stream continuously, and conversation state includes explicit thread branching and merging?

## Success Criteria

- [ ] Candidate API shape covers session lifecycle, turn submission, event streaming, and trace replay
- [ ] Streaming protocol decision (SSE vs WebSocket) is justified against the event flow model
- [ ] Visualization data model maps TraceRecord DAG to renderable nodes and edges
- [ ] API design respects hexagonal architecture (HTTP adapter, not new application layer)

## Open Questions

- Should the web server share the same MechSuitService instance as the CLI, or run as a separate process?
- How should concurrent sessions be managed when the local model can only serve one inference at a time?
- What authentication/authorization model (if any) is appropriate for a local-first tool?
- Should trace replay support incremental fetching (cursor-based) for large conversation histories?
