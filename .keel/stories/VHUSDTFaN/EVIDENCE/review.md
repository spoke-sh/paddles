Runtime event presentation no longer lives in `domain/model`.

What changed:
- Moved `runtime_events.rs` to `src/infrastructure/runtime_presentation.rs`.
- Removed `pub mod runtime_events;` and the runtime presentation re-exports from `src/domain/model/mod.rs`.
- Updated the TUI and web adapters to import `RuntimeEventPresentation`, `project_runtime_event`, and `project_runtime_event_for_tui` from the infrastructure boundary.

Why this satisfies the story:
- Domain event types remain typed domain facts in `TurnEvent` and related model types.
- Surface-facing badges, titles, detail strings, and verbosity shaping are now owned by the outer adapter layer.
- Existing projector tests and targeted TUI/web contract tests continue to pass against the new ownership boundary.
