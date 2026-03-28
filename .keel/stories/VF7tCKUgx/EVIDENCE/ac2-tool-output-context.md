# AC2 Proof: Tool Outputs Recorded As Searchable Context

- `SiftAgentAdapter::respond` pushes each executed tool result into `LocalContextSource::ToolOutput` via `ToolOutputInput::new(...)`.
- The adapter persists that local context in `SessionState.local_context` for later turns.
- `respond_records_tool_outputs_and_turns_in_local_context` verifies that tool output content is stored in local context after a tool-assisted response.
- `tool_failures_are_recorded_and_can_recover` verifies failed tool output is also recorded for later recovery.
- `search_tool_uses_sift_context_assembly` verifies the runtime uses Sift context assembly so recorded context remains part of the searchable turn state.
