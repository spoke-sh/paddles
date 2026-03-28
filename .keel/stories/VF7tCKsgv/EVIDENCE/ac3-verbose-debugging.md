# AC3 Proof: Verbose Runtime Debugging

- `src/main.rs` emits boot and registry synchronization logs under `-v`.
- `src/infrastructure/adapters/sift_agent.rs` emits:
  - `[INFO]` messages when sending prompts and executing tools
  - `[DEBUG]` model responses
  - `[TRACE]` prompt payloads
  - context assembly summaries showing hits, retained artifacts, and pruning
- The log points cover the controller path end to end: CLI boot, context assembly, tool execution, and model response inspection.
