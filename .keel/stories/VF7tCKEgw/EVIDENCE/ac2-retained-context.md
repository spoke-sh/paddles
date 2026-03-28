# AC2 Proof: Retained Context State

- `src/infrastructure/adapters/sift_agent.rs` persists `retained_artifacts` and `local_context` in `SessionState` across calls to `respond`.
- The adapter records user turns, assistant turns, and tool results as `LocalContextSource::{AgentTurn,ToolOutput}` before the next turn.
- `cargo test` covers this behavior with:
  - `respond_records_tool_outputs_and_turns_in_local_context`
  - `search_tool_uses_sift_context_assembly`
  - `tool_failures_are_recorded_and_can_recover`
- Additional regression coverage now ensures workspace tools do not escape via symlinks and that failed `shell`/`apply_patch` commands surface as tool failures.
